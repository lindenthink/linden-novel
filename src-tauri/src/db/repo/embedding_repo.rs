use sqlx::SqlitePool;

use crate::db::pool;
use crate::models::embedding::{
    EmbeddingRow, EmbeddingSourceType, RetrievedItem, UpsertEmbedding,
};

/// 将 f32 切片序列化为 little-endian bytes
fn encode_embedding(values: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(values.len() * 4);
    for v in values {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    bytes
}

/// 将 little-endian bytes 反序列化为 f32 向量
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

/// 计算两个向量的余弦相似度
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

/// 插入或更新嵌入（UPSERT）
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
    Ok(())
}

/// 获取单个嵌入（含反序列化后的向量）
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

/// 列出某项目下所有嵌入（用于内存搜索）
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

/// 按来源删除
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
    Ok(())
}

/// 按项目删除全部（项目删除时级联）
pub async fn delete_by_project(pool: &SqlitePool, project_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM embeddings WHERE project_id = ?")
        .bind(project_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// 向量搜索：返回与 query_embedding 最相似的 top_k 个条目
///
/// 实现：加载项目全部嵌入到内存，逐个计算余弦相似度，排序取 top_k。
/// 适用于摘要级粒度（每项目约 100-250 向量），无需 sqlite-vec 虚拟表。
pub async fn search(
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

    // 按分数降序，取 top_k
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
