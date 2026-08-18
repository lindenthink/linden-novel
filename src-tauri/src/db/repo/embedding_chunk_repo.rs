use sqlx::SqlitePool;

use crate::db::pool;
use crate::models::embedding_chunk::{EmbeddingChunkRow, RetrievedChunk, UpsertChunk};

/// 复用 embedding_repo 中已定义的 encode/decode/cosine
fn encode_embedding(values: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(values.len() * 4);
    for v in values {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    bytes
}

fn decode_embedding(bytes: &[u8], dim: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(dim);
    for i in 0..dim {
        let start = i * 4;
        if start + 4 > bytes.len() {
            break;
        }
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&bytes[start..start + 4]);
        out.push(f32::from_le_bytes(buf));
    }
    out
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;
    for i in 0..n {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 {
        0.0
    } else {
        dot / denom
    }
}

/// 用事务替换某章节所有切片（原子性：先删旧 → 插新）
/// 同时尝试同步 chunks_vec（失败仅告警）
pub async fn replace_chapter_chunks(
    pool: &SqlitePool,
    chunks: Vec<UpsertChunk>,
) -> Result<usize, sqlx::Error> {
    if chunks.is_empty() {
        return Ok(0);
    }
    let chapter_id = chunks[0].chapter_id.clone();
    let ts = pool::now();

    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM embedding_chunks WHERE chapter_id = ?")
        .bind(&chapter_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM chunks_vec WHERE chapter_id = ?")
        .bind(&chapter_id)
        .execute(&mut *tx)
        .await
        .ok(); // 忽略（表不存在的情况）

    let mut count = 0usize;
    for chunk in &chunks {
        let id = uuid::Uuid::new_v4().to_string();
        let bytes = encode_embedding(&chunk.embedding);
        let dim = chunk.embedding.len() as i64;
        let char_count = chunk.chunk_text.chars().count() as i64;

        sqlx::query(
            "INSERT INTO embedding_chunks
                (id, project_id, chapter_id, chunk_index, chunk_text, char_count, content_hash, embedding, dim, model, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&chunk.project_id)
        .bind(&chunk.chapter_id)
        .bind(chunk.chunk_index as i32)
        .bind(&chunk.chunk_text)
        .bind(char_count)
        .bind(&chunk.content_hash)
        .bind(&bytes)
        .bind(dim)
        .bind(&chunk.model)
        .bind(&ts)
        .bind(&ts)
        .execute(&mut *tx)
        .await?;

        // 同步 chunks_vec（维度不匹配时静默失败）
        let _ = sqlx::query(
            "INSERT INTO chunks_vec (project_id, chapter_id, chunk_index, embedding) VALUES (?, ?, ?, ?)",
        )
        .bind(&chunk.project_id)
        .bind(&chunk.chapter_id)
        .bind(chunk.chunk_index as i32)
        .bind(&bytes)
        .execute(&mut *tx)
        .await;

        count += 1;
    }

    tx.commit().await?;
    Ok(count)
}

pub async fn delete_by_chapter(pool: &SqlitePool, chapter_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM embedding_chunks WHERE chapter_id = ?")
        .bind(chapter_id)
        .execute(pool)
        .await?;
    let _ = sqlx::query("DELETE FROM chunks_vec WHERE chapter_id = ?")
        .bind(chapter_id)
        .execute(pool)
        .await;
    Ok(())
}

pub async fn delete_by_project(pool: &SqlitePool, project_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM embedding_chunks WHERE project_id = ?")
        .bind(project_id)
        .execute(pool)
        .await?;
    let _ = sqlx::query("DELETE FROM chunks_vec WHERE project_id = ?")
        .bind(project_id)
        .execute(pool)
        .await;
    Ok(())
}

/// 删除指定卷下所有章节的切片向量
///
/// 必须在卷被级联删除（chapters 随之删除）之前调用，否则子查询将查不到章节。
pub async fn delete_by_volume(pool: &SqlitePool, volume_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "DELETE FROM embedding_chunks WHERE chapter_id IN (SELECT id FROM chapters WHERE volume_id = ?)",
    )
    .bind(volume_id)
    .execute(pool)
    .await?;
    let _ = sqlx::query(
        "DELETE FROM chunks_vec WHERE chapter_id IN (SELECT id FROM chapters WHERE volume_id = ?)",
    )
    .bind(volume_id)
    .execute(pool)
    .await;
    Ok(())
}

pub async fn list_chapter_hashes(
    pool: &SqlitePool,
    chapter_id: &str,
) -> Result<Vec<(i32, String)>, sqlx::Error> {
    let rows: Vec<(i32, String)> = sqlx::query_as(
        "SELECT chunk_index, content_hash FROM embedding_chunks WHERE chapter_id = ? ORDER BY chunk_index",
    )
    .bind(chapter_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// 切片搜索：优先 chunks_vec，失败内存回退
pub async fn search(
    pool: &SqlitePool,
    project_id: &str,
    query_embedding: &[f32],
    top_k: usize,
    exclude_chapter_ids: &[String],
) -> Result<Vec<RetrievedChunk>, sqlx::Error> {
    let start = std::time::Instant::now();
    match search_via_vec0(pool, project_id, query_embedding, top_k, exclude_chapter_ids).await {
        Ok(r) => {
            tracing::debug!(
                "chunks_vec hit: {} results, {:?}",
                r.len(),
                start.elapsed()
            );
            Ok(r)
        }
        Err(e) => {
            tracing::warn!(
                "chunks_vec search failed, fallback to memory (slow): {} ({:?})",
                e,
                start.elapsed()
            );
            search_via_memory(pool, project_id, query_embedding, top_k, exclude_chapter_ids).await
        }
    }
}

async fn search_via_vec0(
    pool: &SqlitePool,
    project_id: &str,
    query_embedding: &[f32],
    top_k: usize,
    exclude_chapter_ids: &[String],
) -> Result<Vec<RetrievedChunk>, sqlx::Error> {
    let query_bytes = encode_embedding(query_embedding);
    // project_id 作为 partition key 可在 KNN WHERE 过滤；
    // exclude_chapter_ids 仍是多值过滤，partition key 只支持单值 `=`，需过取后在 Rust 层过滤
    let over_fetch = top_k * 3;
    let sql = format!(
        "SELECT chapter_id, chunk_index, distance
         FROM chunks_vec
         WHERE project_id = ?
           AND embedding MATCH ?
           AND k = {}",
        over_fetch
    );

    let rows: Vec<(String, i32, f32)> = sqlx::query_as(&sql)
        .bind(project_id)
        .bind(&query_bytes)
        .fetch_all(pool)
        .await?;

    let mut scored: Vec<RetrievedChunk> = rows
        .into_iter()
        // Rust 层仅过滤 exclude_chapter_ids（partition key 已在 SQL 层过滤 project_id）
        .filter(|(cid, _, _)| !exclude_chapter_ids.iter().any(|ex| ex == cid))
        .map(|(chapter_id, chunk_index, dist)| RetrievedChunk {
            chapter_id,
            chunk_index,
            chunk_text: String::new(), // 占位，下方回查
            score: 1.0 - dist,
        })
        .collect();

    // 取 top_k，然后补 chunk_text 内容（批量查询避免 N+1）
    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(top_k);

    if !scored.is_empty() {
        // 用行值表达式批量查询：(chapter_id, chunk_index) IN ((?,?), (?,?), ...)
        let placeholders: Vec<String> = scored
            .iter()
            .map(|_| "(?, ?)".to_string())
            .collect();
        let sql = format!(
            "SELECT chapter_id, chunk_index, chunk_text FROM embedding_chunks \
             WHERE (chapter_id, chunk_index) IN ({})",
            placeholders.join(", ")
        );
        let mut q = sqlx::query_as::<_, (String, i32, String)>(&sql);
        for item in &scored {
            q = q.bind(item.chapter_id.clone()).bind(item.chunk_index);
        }
        let rows: Vec<(String, i32, String)> = q.fetch_all(pool).await?;

        // 建立 (chapter_id, chunk_index) -> chunk_text 映射后批量回填
        let mut text_map: std::collections::HashMap<(String, i32), String> =
            std::collections::HashMap::with_capacity(rows.len());
        for (cid, idx, text) in rows {
            text_map.insert((cid, idx), text);
        }
        for item in &mut scored {
            if let Some(text) = text_map.get(&(item.chapter_id.clone(), item.chunk_index)) {
                item.chunk_text = text.clone();
            }
        }
    }

    Ok(scored)
}

async fn search_via_memory(
    pool: &SqlitePool,
    project_id: &str,
    query_embedding: &[f32],
    top_k: usize,
    exclude_chapter_ids: &[String],
) -> Result<Vec<RetrievedChunk>, sqlx::Error> {
    let rows = sqlx::query_as::<_, EmbeddingChunkRow>(
        "SELECT * FROM embedding_chunks WHERE project_id = ?",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    let mut scored: Vec<RetrievedChunk> = rows
        .into_iter()
        .filter(|r| !exclude_chapter_ids.iter().any(|ex| *ex == r.chapter_id))
        .map(|r| {
            let dim = r.dim as usize;
            let vec = decode_embedding(&r.embedding, dim);
            RetrievedChunk {
                chapter_id: r.chapter_id,
                chunk_index: r.chunk_index,
                chunk_text: r.chunk_text,
                score: cosine_similarity(query_embedding, &vec),
            }
        })
        .collect();

    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(top_k);
    Ok(scored)
}
