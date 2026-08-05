use sqlx::SqlitePool;
use crate::error::AppError;
use crate::models::ai_generation::{AiGenerationHistory, CreateAiGeneration};

pub async fn create(
    pool: &SqlitePool,
    input: &CreateAiGeneration,
) -> Result<AiGenerationHistory, AppError> {
    let id = uuid::Uuid::new_v4().to_string();
    let parameters_json = serde_json::to_string(&input.parameters)
        .map_err(|e| AppError::Internal(format!("Failed to serialize parameters: {}", e)))?;

    sqlx::query_as::<_, AiGenerationHistory>(
        r#"
        INSERT INTO ai_generation_history (id, chapter_id, mode, input_context, output_content, parameters_json)
        VALUES (?, ?, ?, ?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(&id)
    .bind(&input.chapter_id)
    .bind(&input.mode)
    .bind(&input.input_context)
    .bind(&input.output_content)
    .bind(&parameters_json)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to create generation history: {}", e)))
}

pub async fn list_by_chapter(
    pool: &SqlitePool,
    chapter_id: &str,
) -> Result<Vec<AiGenerationHistory>, AppError> {
    sqlx::query_as::<_, AiGenerationHistory>(
        r#"
        SELECT * FROM ai_generation_history
        WHERE chapter_id = ?
        ORDER BY created_at DESC
        "#,
    )
    .bind(chapter_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to list generation history: {}", e)))
}

pub async fn get(
    pool: &SqlitePool,
    id: &str,
) -> Result<AiGenerationHistory, AppError> {
    sqlx::query_as::<_, AiGenerationHistory>(
        r#"
        SELECT * FROM ai_generation_history
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to get generation history: {}", e)))
}

pub async fn delete(
    pool: &SqlitePool,
    id: &str,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM ai_generation_history WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to delete generation history: {}", e)))?;
    Ok(())
}

pub async fn delete_by_chapter(
    pool: &SqlitePool,
    chapter_id: &str,
) -> Result<u64, AppError> {
    let result = sqlx::query("DELETE FROM ai_generation_history WHERE chapter_id = ?")
        .bind(chapter_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to delete generation history: {}", e)))?;
    Ok(result.rows_affected())
}
