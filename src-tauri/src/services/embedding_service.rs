use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

use crate::ai::provider::{EmbeddingRequest, EmbeddingResponse};
use crate::ai::provider_factory;
use crate::db::repo::{embedding_chunk_repo, embedding_repo};
use crate::error::AppError;
use crate::models::embedding::{EmbeddingSourceType, UpsertEmbedding};

/// 计算文本内容的 SHA-256 哈希（十六进制小写）
fn content_hash(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// 解析 source_type 字符串为枚举
fn parse_source_type(s: &str) -> Result<EmbeddingSourceType, AppError> {
    s.parse::<EmbeddingSourceType>()
        .map_err(|e: String| AppError::Validation(e))
}

/// 为指定内容生成并存储嵌入向量
///
/// # 参数
/// - `pool`: 数据库连接池
/// - `app_data_dir`: 应用数据目录（用于密钥解密）
/// - `project_id`: 项目 ID
/// - `source_type`: 嵌入来源类型（chapter/character/storyline/worldview）
/// - `source_id`: 来源 ID
/// - `content`: 要嵌入的文本内容
/// - `embedding_model`: 可选的嵌入模型名（空字符串使用默认）
///
/// # 行为
/// - 计算 content_hash，若已存在且 hash 未变则跳过
/// - 调用 AI Provider 生成嵌入
/// - UPSERT 存入 embeddings 表
pub async fn generate_and_store(
    pool: &SqlitePool,
    app_data_dir: &std::path::Path,
    project_id: &str,
    source_type: &str,
    source_id: &str,
    content: &str,
    embedding_model: &str,
) -> Result<bool, AppError> {
    let st = parse_source_type(source_type)?;
    let hash = content_hash(content);

    // 变更检测：hash 未变则跳过
    if let Some((row, _)) = embedding_repo::get_by_source(pool, st, source_id).await? {
        if row.content_hash == hash {
            tracing::debug!(
                "Embedding unchanged for {}:{} (hash={}), skipping",
                source_type,
                source_id,
                &hash[..16]
            );
            return Ok(false);
        }
    }

    // 空内容跳过
    if content.trim().is_empty() {
        return Ok(false);
    }

    // 获取本地嵌入 provider 并生成嵌入
    let provider = provider_factory::get_local_embedder(app_data_dir)?;

    let request = EmbeddingRequest {
        model: embedding_model.to_string(),
        input: content.to_string(),
    };

    let response: EmbeddingResponse = provider.embed(request).await?;
    let dim = response.dim;
    let model_name = response.model.clone();

    // 存储
    let input = UpsertEmbedding {
        project_id: project_id.to_string(),
        source_type: st,
        source_id: source_id.to_string(),
        content_hash: hash,
        embedding: response.vector,
        model: response.model,
    };

    embedding_repo::upsert(pool, &input).await?;

    tracing::info!(
        "Embedded {}:{} dim={} model={}",
        source_type,
        source_id,
        dim,
        model_name
    );

    Ok(true)
}

/// 删除指定项目的全部嵌入
pub async fn remove_by_project(pool: &SqlitePool, project_id: &str) -> Result<(), AppError> {
    embedding_repo::delete_by_project(pool, project_id).await?;
    // 同步清理切片级向量（embedding_chunks + chunks_vec），避免孤儿数据
    embedding_chunk_repo::delete_by_project(pool, project_id).await?;
    Ok(())
}

/// 向量搜索（直接接受 query 向量）
///
/// 调用方负责先 embed query 文本，RAG 流程中复用同一向量给摘要级 + 切片级检索
pub async fn search_by_vector(
    pool: &SqlitePool,
    project_id: &str,
    query_embedding: &[f32],
    top_k: usize,
) -> Result<Vec<crate::models::embedding::RetrievedItem>, AppError> {
    let results = embedding_repo::search(pool, project_id, query_embedding, top_k).await?;
    Ok(results)
}

/// 批量嵌入处理的统计结果
#[derive(Debug, Default, serde::Serialize)]
pub struct SyncEmbeddingsResult {
    pub chapter_summary_embedded: usize,
    pub character_embedded: usize,
    pub storyline_embedded: usize,
    pub worldview_embedded: usize,
    pub chunk_embedded: usize,
    pub total_embedded: usize,
}

/// 为项目的所有章节摘要 + 元素描述批量生成嵌入
///
/// # 参数
/// - `progress`: 进度回调 `(current, total)`，current 为已完成的"步骤"数，total 为总步骤数
///   - 步骤定义：5 个大类，每个大类处理完所有条目算 1 步（摘要/角色/故事线/世界观/切片）
///   - 即 total 恒等于 5，每个步骤完成时 current+1
pub async fn sync_project_embeddings<F>(
    pool: &SqlitePool,
    app_data_dir: &std::path::Path,
    project_id: &str,
    mut progress: F,
) -> Result<SyncEmbeddingsResult, AppError>
where
    F: FnMut(usize, usize) + Send,
{
    let mut result = SyncEmbeddingsResult::default();
    let total_steps = 5usize;

    // 0/5 初始
    progress(0, total_steps);

    // 1. 章节摘要
    let chapters = sqlx::query_as::<_, crate::models::chapter::Chapter>(
        "SELECT * FROM chapters WHERE project_id = ? AND summary IS NOT NULL AND summary != ''",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    for chapter in &chapters {
        if let Some(summary) = &chapter.summary {
            if generate_and_store(
                pool,
                app_data_dir,
                project_id,
                "chapter",
                &chapter.id,
                summary,
                "",
            )
            .await?
            {
                result.chapter_summary_embedded += 1;
            }
        }
    }
    progress(1, total_steps);

    // 2. 角色描述
    let characters = sqlx::query_as::<_, crate::models::character::Character>(
        "SELECT * FROM characters WHERE project_id = ? AND description IS NOT NULL AND description != ''",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    for character in &characters {
        if let Some(desc) = &character.description {
            if generate_and_store(
                pool,
                app_data_dir,
                project_id,
                "character",
                &character.id,
                desc,
                "",
            )
            .await?
            {
                result.character_embedded += 1;
            }
        }
    }
    progress(2, total_steps);

    // 3. 故事线描述
    let storylines = sqlx::query_as::<_, crate::models::storyline::Storyline>(
        "SELECT * FROM storylines WHERE project_id = ? AND description IS NOT NULL AND description != ''",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    for sl in &storylines {
        if let Some(desc) = &sl.description {
            if generate_and_store(
                pool,
                app_data_dir,
                project_id,
                "storyline",
                &sl.id,
                desc,
                "",
            )
            .await?
            {
                result.storyline_embedded += 1;
            }
        }
    }
    progress(3, total_steps);

    // 4. 世界观描述
    let worldviews = sqlx::query_as::<_, crate::models::worldview::WorldviewEntry>(
        "SELECT * FROM worldview WHERE project_id = ? AND description IS NOT NULL AND description != ''",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    for wv in &worldviews {
        if let Some(desc) = &wv.description {
            if generate_and_store(
                pool,
                app_data_dir,
                project_id,
                "worldview",
                &wv.id,
                desc,
                "",
            )
            .await?
            {
                result.worldview_embedded += 1;
            }
        }
    }
    progress(4, total_steps);

    // 5. 章节正文切片嵌入（细粒度 RAG，失败仅告警）
    match crate::services::chunk_embedding_service::embed_project(pool, app_data_dir, project_id).await {
        Ok(n) => {
            result.chunk_embedded = n;
            tracing::info!("Chunk embeddings synced: {} slices", n);
        }
        Err(e) => {
            tracing::warn!("Chunk embedding project {} skipped: {}", project_id, e);
        }
    }

    result.total_embedded = result.chapter_summary_embedded
        + result.character_embedded
        + result.storyline_embedded
        + result.worldview_embedded
        + result.chunk_embedded;

    progress(5, total_steps);
    Ok(result)
}

/// 向后兼容的简化封装（无进度回调场景）
pub async fn sync_project_embeddings_silent(
    pool: &SqlitePool,
    app_data_dir: &std::path::Path,
    project_id: &str,
) -> Result<usize, AppError> {
    let r = sync_project_embeddings(pool, app_data_dir, project_id, |_, _| {}).await?;
    Ok(r.total_embedded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_hash_deterministic() {
        let h1 = content_hash("hello world");
        let h2 = content_hash("hello world");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // 32 bytes hex = 64 chars
    }

    #[test]
    fn test_content_hash_different() {
        let h1 = content_hash("hello");
        let h2 = content_hash("world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_content_hash_empty() {
        let h = content_hash("");
        assert_eq!(h.len(), 64);
    }

    #[test]
    fn test_parse_source_type_valid() {
        assert!(parse_source_type("chapter").is_ok());
        assert!(parse_source_type("character").is_ok());
        assert!(parse_source_type("storyline").is_ok());
        assert!(parse_source_type("worldview").is_ok());
    }

    #[test]
    fn test_parse_source_type_invalid() {
        assert!(parse_source_type("unknown").is_err());
        assert!(parse_source_type("").is_err());
    }
}
