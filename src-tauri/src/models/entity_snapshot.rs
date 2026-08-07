use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 实体类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum EntityType {
    Character,
    Storyline,
}

impl EntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Character => "character",
            Self::Storyline => "storyline",
        }
    }
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for EntityType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "character" => Ok(Self::Character),
            "storyline" => Ok(Self::Storyline),
            other => Err(format!("unknown entity type: {}", other)),
        }
    }
}

/// 数据库行
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct EntitySnapshot {
    pub id: String,
    pub project_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub chapter_id: String,
    pub state_json: String,
    pub summary: String,
    pub changes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// 创建/更新快照
#[derive(Debug, Clone)]
pub struct UpsertEntitySnapshot {
    pub project_id: String,
    pub entity_type: EntityType,
    pub entity_id: String,
    pub chapter_id: String,
    pub state_json: String,
    pub summary: String,
    pub changes: Option<String>,
}

/// 实体演变历史
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityEvolution {
    pub entity_id: String,
    pub entity_type: String,
    pub name: String,
    pub snapshots: Vec<EntitySnapshotWithChapter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySnapshotWithChapter {
    pub snapshot: EntitySnapshot,
    pub chapter_title: String,
    pub order_index: i32,
}

// 手动实现 FromRow 以支持 JOIN 查询
impl<'r> FromRow<'r, sqlx::sqlite::SqliteRow> for EntitySnapshotWithChapter {
    fn from_row(
        row: &'r sqlx::sqlite::SqliteRow,
    ) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(Self {
            snapshot: EntitySnapshot {
                id: row.try_get("id")?,
                project_id: row.try_get("project_id")?,
                entity_type: row.try_get("entity_type")?,
                entity_id: row.try_get("entity_id")?,
                chapter_id: row.try_get("chapter_id")?,
                state_json: row.try_get("state_json")?,
                summary: row.try_get("summary")?,
                changes: row.try_get("changes")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            },
            chapter_title: row.try_get("chapter_title")?,
            order_index: row.try_get("order_index")?,
        })
    }
}
