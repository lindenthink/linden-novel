use sqlx::SqlitePool;

use crate::db::pool;
use crate::models::entity_snapshot::{
    EntitySnapshot, EntitySnapshotWithChapter, EntityType, EntityEvolution, ProjectEntity, UpsertEntitySnapshot,
};

/// 插入或更新快照（UPSERT）
pub async fn upsert(
    pool: &SqlitePool,
    input: &UpsertEntitySnapshot,
) -> Result<(), sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let ts = pool::now();

    sqlx::query(
        "INSERT INTO entity_snapshots (id, project_id, entity_type, entity_id, chapter_id, state_json, summary, changes, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(entity_type, entity_id, chapter_id) DO UPDATE SET
            state_json = excluded.state_json,
            summary    = excluded.summary,
            changes    = excluded.changes,
            updated_at = excluded.updated_at",
    )
    .bind(&id)
    .bind(&input.project_id)
    .bind(input.entity_type.as_str())
    .bind(&input.entity_id)
    .bind(&input.chapter_id)
    .bind(&input.state_json)
    .bind(&input.summary)
    .bind(&input.changes)
    .bind(&ts)
    .bind(&ts)
    .execute(pool)
    .await?;
    Ok(())
}

/// 列出某实体的全部快照（按章节 order_index 排序）
pub async fn list_by_entity(
    pool: &SqlitePool,
    entity_type: EntityType,
    entity_id: &str,
) -> Result<Vec<EntitySnapshotWithChapter>, sqlx::Error> {
    sqlx::query_as::<_, EntitySnapshotWithChapter>(
        "SELECT s.id, s.project_id, s.entity_type, s.entity_id, s.chapter_id,
                s.state_json, s.summary, s.changes, s.created_at, s.updated_at,
                c.title AS chapter_title, c.order_index
         FROM entity_snapshots s
         JOIN chapters c ON c.id = s.chapter_id
         WHERE s.entity_type = ? AND s.entity_id = ?
         ORDER BY c.order_index ASC",
    )
    .bind(entity_type.as_str())
    .bind(entity_id)
    .fetch_all(pool)
    .await
}

/// 列出某章节的全部快照
pub async fn list_by_chapter(
    pool: &SqlitePool,
    chapter_id: &str,
) -> Result<Vec<EntitySnapshot>, sqlx::Error> {
    sqlx::query_as::<_, EntitySnapshot>(
        "SELECT * FROM entity_snapshots WHERE chapter_id = ?",
    )
    .bind(chapter_id)
    .fetch_all(pool)
    .await
}

/// 判断指定章节是否已存在任意实体快照（批量生成时用于跳过已有快照的章节）
pub async fn exists_by_chapter_id(
    pool: &SqlitePool,
    chapter_id: &str,
) -> Result<bool, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM entity_snapshots WHERE chapter_id = ?",
    )
    .bind(chapter_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0 > 0)
}

/// 列出项目内所有有快照的实体（去重），返回 (entity_type, entity_id, name)
pub async fn list_project_entities(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<Vec<ProjectEntity>, sqlx::Error> {
    // 角色
    let mut characters: Vec<ProjectEntity> = sqlx::query_as(
        "SELECT DISTINCT s.entity_type AS entity_type, s.entity_id AS entity_id, c.name AS name
         FROM entity_snapshots s
         JOIN characters c ON c.id = s.entity_id
         WHERE s.project_id = ? AND s.entity_type = 'character'",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    // 故事线
    let mut storylines: Vec<ProjectEntity> = sqlx::query_as(
        "SELECT DISTINCT s.entity_type AS entity_type, s.entity_id AS entity_id, st.name AS name
         FROM entity_snapshots s
         JOIN storylines st ON st.id = s.entity_id
         WHERE s.project_id = ? AND s.entity_type = 'storyline'",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    characters.append(&mut storylines);
    Ok(characters)
}

/// 获取实体演变历史（含章节信息）
pub async fn get_evolution(
    pool: &SqlitePool,
    entity_type: EntityType,
    entity_id: &str,
) -> Result<EntityEvolution, sqlx::Error> {
    let snapshots = list_by_entity(pool, entity_type, entity_id).await?;

    // 获取实体名称
    let name = match entity_type {
        EntityType::Character => {
            let row: Option<(String,)> =
                sqlx::query_as("SELECT name FROM characters WHERE id = ?")
                    .bind(entity_id)
                    .fetch_optional(pool)
                    .await?;
            row.map(|(n,)| n).unwrap_or_else(|| entity_id.to_string())
        }
        EntityType::Storyline => {
            let row: Option<(String,)> =
                sqlx::query_as("SELECT name FROM storylines WHERE id = ?")
                    .bind(entity_id)
                    .fetch_optional(pool)
                    .await?;
            row.map(|(n,)| n).unwrap_or_else(|| entity_id.to_string())
        }
    };

    Ok(EntityEvolution {
        entity_id: entity_id.to_string(),
        entity_type: entity_type.as_str().to_string(),
        name,
        snapshots,
    })
}

/// 按章节删除
pub async fn delete_by_chapter(
    pool: &SqlitePool,
    chapter_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM entity_snapshots WHERE chapter_id = ?")
        .bind(chapter_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// 按实体删除
pub async fn delete_by_entity(
    pool: &SqlitePool,
    entity_type: EntityType,
    entity_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM entity_snapshots WHERE entity_type = ? AND entity_id = ?")
        .bind(entity_type.as_str())
        .bind(entity_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// 按项目删除
pub async fn delete_by_project(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM entity_snapshots WHERE project_id = ?")
        .bind(project_id)
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_type_as_str() {
        assert_eq!(EntityType::Character.as_str(), "character");
        assert_eq!(EntityType::Storyline.as_str(), "storyline");
    }

    #[test]
    fn test_entity_type_from_str() {
        assert!("character".parse::<EntityType>().is_ok());
        assert!("storyline".parse::<EntityType>().is_ok());
        assert!("unknown".parse::<EntityType>().is_err());
    }
}
