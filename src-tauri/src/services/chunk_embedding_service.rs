use sqlx::SqlitePool;
use std::path::Path;

use crate::ai::chunker::{chunk_text, ChunkConfig};
use crate::ai::provider::{BatchEmbeddingRequest, EmbeddingRequest};
use crate::ai::provider_factory;
use crate::db::repo::embedding_chunk_repo;
use crate::error::AppError;
use crate::models::embedding_chunk::{RetrievedChunk, UpsertChunk};

/// 为单个章节生成切片嵌入
///
/// 流程：切片 → 哈希对比（筛未变切片）→ 批量嵌入新切片 → 替换存储
pub async fn embed_chapter(
    pool: &SqlitePool,
    app_data_dir: &Path,
    chapter_id: &str,
    project_id: &str,
    content_text: &str,
    config: &ChunkConfig,
) -> Result<usize, AppError> {
    let chunks = chunk_text(content_text, config);
    if chunks.is_empty() {
        embedding_chunk_repo::delete_by_chapter(pool, chapter_id).await?;
        return Ok(0);
    }

    // 变更检测：筛出需要重新嵌入的切片
    let existing = embedding_chunk_repo::list_chapter_hashes(pool, chapter_id).await?;
    let existing_map: std::collections::HashMap<i32, String> = existing.into_iter().collect();
    let need_embed_indices: Vec<usize> = chunks
        .iter()
        .filter(|c| existing_map.get(&(c.index as i32)) != Some(&c.content_hash))
        .map(|c| c.index)
        .collect();

    // 获取本地嵌入 provider
    if need_embed_indices.is_empty() {
        tracing::debug!("Chapter {} chunks unchanged, skip embed", chapter_id);
        return Ok(0);
    }
    let provider = provider_factory::get_local_embedder(app_data_dir)?;

    // 批量嵌入：仅需要变化的切片
    let inputs: Vec<String> = need_embed_indices
        .iter()
        .map(|i| chunks[*i].text.clone())
        .collect();
    let batch_resp = provider
        .embed_batch(BatchEmbeddingRequest {
            model: String::new(),
            inputs,
        })
        .await?;

    let mut vector_iter = batch_resp.vectors.into_iter();

    // 组装 upserts：变化切片用新向量；未变切片给占位（向量值为空）
    //
    // 注意：replace_chapter_chunks 是"全量替换"语义，所以未变切片也必须带进来，
    // 但其向量我们可以从 DB 读回重用以避免浪费。
    // 开发阶段为了简单，直接对未变切片再跑一次单条 embed 也可以。
    // 下面用更经济的方案：变化切片从 batch_resp 取，未变切片走单条 DB 读回向量。
    let mut upserts: Vec<UpsertChunk> = Vec::with_capacity(chunks.len());
    for chunk in &chunks {
        let embedding = if need_embed_indices.contains(&chunk.index) {
            vector_iter.next().unwrap_or_default()
        } else {
            // 未变切片：从 DB 读回已有向量避免重嵌入
            read_existing_vector(pool, chapter_id, chunk.index).await
        };
        upserts.push(UpsertChunk {
            project_id: project_id.to_string(),
            chapter_id: chapter_id.to_string(),
            chunk_index: chunk.index,
            chunk_text: chunk.text.clone(),
            content_hash: chunk.content_hash.clone(),
            embedding,
            model: batch_resp.model.clone(),
        });
    }

    let count = embedding_chunk_repo::replace_chapter_chunks(pool, upserts).await?;
    tracing::info!(
        "Chapter {}: {} chunks embedded ({} changed)",
        chapter_id,
        count,
        need_embed_indices.len()
    );
    Ok(count)
}

/// 从 DB 读回已存切片的向量（若不存在则返回空向量，上层不会用到这种异常情况）
async fn read_existing_vector(pool: &SqlitePool, chapter_id: &str, chunk_index: usize) -> Vec<f32> {
    use sqlx::Row;
    let row = sqlx::query(
        "SELECT embedding, dim FROM embedding_chunks WHERE chapter_id = ? AND chunk_index = ?",
    )
    .bind(chapter_id)
    .bind(chunk_index as i32)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    match row {
        Some(r) => {
            let bytes: Vec<u8> = r.try_get("embedding").unwrap_or_default();
            let dim: i64 = r.try_get("dim").unwrap_or(0);
            if bytes.is_empty() || dim == 0 {
                Vec::new()
            } else {
                let mut out = Vec::with_capacity(dim as usize);
                for i in 0..dim as usize {
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
        }
        None => Vec::new(),
    }
}

/// 项目级批量切片嵌入
pub async fn embed_project(
    pool: &SqlitePool,
    app_data_dir: &Path,
    project_id: &str,
) -> Result<usize, AppError> {
    let config = ChunkConfig::default();
    let mut total = 0usize;

    let chapters: Vec<(String, String)> = sqlx::query_as(
        "SELECT c.id, ct.content_text
         FROM chapters c
         JOIN chapter_contents ct ON ct.chapter_id = c.id
         WHERE c.project_id = ?
           AND ct.content_text IS NOT NULL
           AND ct.content_text != ''",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    for (chapter_id, content_text) in chapters {
        match embed_chapter(pool, app_data_dir, &chapter_id, project_id, &content_text, &config)
            .await
        {
            Ok(n) => total += n,
            Err(e) => tracing::warn!("Chunk embed chapter {} failed: {}", chapter_id, e),
        }
    }
    Ok(total)
}

/// 切片级语义检索（对外服务接口）
pub async fn search_chunks(
    pool: &SqlitePool,
    app_data_dir: &Path,
    project_id: &str,
    query: &str,
    top_k: usize,
    exclude_chapter_id: Option<&str>,
) -> Result<Vec<RetrievedChunk>, AppError> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let provider = provider_factory::get_local_embedder(app_data_dir)?;
    let resp = provider
        .embed(EmbeddingRequest {
            model: String::new(),
            input: query.to_string(),
        })
        .await?;

    let results = embedding_chunk_repo::search(
        pool,
        project_id,
        &resp.vector,
        top_k,
        exclude_chapter_id,
    )
    .await?;

    Ok(results)
}
