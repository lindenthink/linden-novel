use sqlx::SqlitePool;

use crate::db::pool;
use crate::models::embedding::{
    EmbeddingRow, EmbeddingSourceType, RetrievedItem, UpsertEmbedding,
};

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

/// UPSERT：同时写入 embeddings 表（权威）+ 同步到 vec0 索引（加速）
///
/// vec0 同步失败仅告警不阻塞：索引损坏可从 embeddings 重建
pub async fn upsert(pool: &SqlitePool, input: &UpsertEmbedding) -> Result<(), sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let ts = pool::now();
    let bytes = encode_embedding(&input.embedding);
    let dim = input.embedding.len() as i64;

    sqlx::query(
        "INSERT INTO embeddings (id, project_id, source_type, source_id, content_hash, embedding, dim, model, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(source_type, source_id) DO UPDATE SET
            content_hash = excluded.content_hash,
            embedding    = excluded.embedding,
            dim          = excluded.dim,
            model        = excluded.model,
            updated_at   = excluded.updated_at",
    )
    .bind(&id)
    .bind(&input.project_id)
    .bind(input.source_type.as_str())
    .bind(&input.source_id)
    .bind(&input.content_hash)
    .bind(&bytes)
    .bind(dim)
    .bind(&input.model)
    .bind(&ts)
    .bind(&ts)
    .execute(pool)
    .await?;

    // 尝试同步 vec0（维度不匹配时静默失败，检索回退内存）
    match sync_to_vec0(pool, input, &bytes).await {
        Ok(_) => {}
        Err(e) => tracing::warn!("embeddings_vec sync skipped: {}", e),
    }

    Ok(())
}

/// 将条目同步到 vec0 虚拟表（先删后插，vec0 不支持 UPSERT）
async fn sync_to_vec0(
    pool: &SqlitePool,
    input: &UpsertEmbedding,
    bytes: &[u8],
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "DELETE FROM embeddings_vec WHERE project_id = ? AND source_type = ? AND source_id = ?",
    )
    .bind(&input.project_id)
    .bind(input.source_type.as_str())
    .bind(&input.source_id)
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO embeddings_vec (project_id, source_type, source_id, embedding) VALUES (?, ?, ?, ?)",
    )
    .bind(&input.project_id)
    .bind(input.source_type.as_str())
    .bind(&input.source_id)
    .bind(bytes)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_by_source(
    pool: &SqlitePool,
    source_type: EmbeddingSourceType,
    source_id: &str,
) -> Result<Option<(EmbeddingRow, Vec<f32>)>, sqlx::Error> {
    let row = sqlx::query_as::<_, EmbeddingRow>(
        "SELECT * FROM embeddings WHERE source_type = ? AND source_id = ?",
    )
    .bind(source_type.as_str())
    .bind(source_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| {
        let dim = r.dim as usize;
        let vec = decode_embedding(&r.embedding, dim);
        (r, vec)
    }))
}

pub async fn list_by_project(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<Vec<(EmbeddingRow, Vec<f32>)>, sqlx::Error> {
    let rows = sqlx::query_as::<_, EmbeddingRow>(
        "SELECT * FROM embeddings WHERE project_id = ?",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let dim = r.dim as usize;
            let vec = decode_embedding(&r.embedding, dim);
            (r, vec)
        })
        .collect())
}

/// 双表同步删除
pub async fn delete_by_source(
    pool: &SqlitePool,
    source_type: EmbeddingSourceType,
    source_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM embeddings WHERE source_type = ? AND source_id = ?")
        .bind(source_type.as_str())
        .bind(source_id)
        .execute(pool)
        .await?;

    // vec0 删除静默失败
    let _ = sqlx::query(
        "DELETE FROM embeddings_vec WHERE source_type = ? AND source_id = ?",
    )
    .bind(source_type.as_str())
    .bind(source_id)
    .execute(pool)
    .await;

    Ok(())
}

/// 双表同步删除项目级
pub async fn delete_by_project(pool: &SqlitePool, project_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM embeddings WHERE project_id = ?")
        .bind(project_id)
        .execute(pool)
        .await?;

    let _ = sqlx::query("DELETE FROM embeddings_vec WHERE project_id = ?")
        .bind(project_id)
        .execute(pool)
        .await;

    Ok(())
}

/// 删除指定卷下所有章节的摘要级嵌入（source_type='chapter'）
///
/// 必须在卷被级联删除（chapters 随之删除）之前调用，否则子查询将查不到章节。
pub async fn delete_chapters_by_volume(
    pool: &SqlitePool,
    volume_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "DELETE FROM embeddings WHERE source_type = 'chapter' AND source_id IN (SELECT id FROM chapters WHERE volume_id = ?)",
    )
    .bind(volume_id)
    .execute(pool)
    .await?;

    let _ = sqlx::query(
        "DELETE FROM embeddings_vec WHERE source_type = 'chapter' AND source_id IN (SELECT id FROM chapters WHERE volume_id = ?)",
    )
    .bind(volume_id)
    .execute(pool)
    .await;

    Ok(())
}

/// 向量搜索：优先 vec0 加速，失败回退内存余弦
pub async fn search(
    pool: &SqlitePool,
    project_id: &str,
    query_embedding: &[f32],
    top_k: usize,
) -> Result<Vec<RetrievedItem>, sqlx::Error> {
    // 优先 vec0 加速，失败回退内存余弦
    let start = std::time::Instant::now();
    match search_via_vec0(pool, project_id, query_embedding, top_k).await {
        Ok(results) => {
            tracing::debug!(
                "embeddings_vec hit: {} results, {:?}",
                results.len(),
                start.elapsed()
            );
            Ok(results)
        }
        Err(e) => {
            tracing::warn!(
                "embeddings_vec search failed, fallback to memory (slow): {} ({:?})",
                e,
                start.elapsed()
            );
            search_via_memory(pool, project_id, query_embedding, top_k).await
        }
    }
}

/// vec0 KNN：project_id 为 partition key，KNN 查询时可直接 WHERE 过滤
///
/// sqlite-vec partition key（`*` 前缀）允许在 KNN 查询中使用 `=` 约束，
/// 语法：`WHERE project_id = ? AND embedding MATCH ? AND k = N`（partition key 在前）。
async fn search_via_vec0(
    pool: &SqlitePool,
    project_id: &str,
    query_embedding: &[f32],
    top_k: usize,
) -> Result<Vec<RetrievedItem>, sqlx::Error> {
    let query_bytes = encode_embedding(query_embedding);
    let sql = format!(
        "SELECT source_type, source_id, distance
         FROM embeddings_vec
         WHERE project_id = ?
           AND embedding MATCH ?
           AND k = {}",
        top_k
    );

    let rows: Vec<(String, String, f32)> = sqlx::query_as(&sql)
        .bind(project_id)
        .bind(&query_bytes)
        .fetch_all(pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(|(st, sid, dist)| RetrievedItem {
            source_type: st,
            source_id: sid,
            score: 1.0 - dist,
        })
        .collect())
}

/// 内存余弦回退
async fn search_via_memory(
    pool: &SqlitePool,
    project_id: &str,
    query_embedding: &[f32],
    top_k: usize,
) -> Result<Vec<RetrievedItem>, sqlx::Error> {
    let all = list_by_project(pool, project_id).await?;
    if all.is_empty() {
        return Ok(Vec::new());
    }

    let mut scored: Vec<RetrievedItem> = all
        .into_iter()
        .map(|(row, vec)| RetrievedItem {
            source_type: row.source_type,
            source_id: row.source_id,
            score: cosine_similarity(query_embedding, &vec),
        })
        .collect();

    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scored.truncate(top_k);
    Ok(scored)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let original = vec![0.1, -0.2, 0.3, 0.0, 1.5];
        let bytes = encode_embedding(&original);
        let decoded = decode_embedding(&bytes, original.len());
        assert_eq!(decoded.len(), original.len());
        for (a, b) in original.iter().zip(decoded.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);

        let c = vec![0.0, 1.0, 0.0];
        assert!(cosine_similarity(&a, &c).abs() < 1e-6);

        let d = vec![-1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &d) + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_empty_vector() {
        assert_eq!(cosine_similarity(&[], &[]), 0.0);
    }
}
