use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::models::{chapter::Chapter, project::Project, volume::Volume};

/// 导出格式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Txt,
    Md,
    Json,
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Txt => write!(f, "txt"),
            Self::Md => write!(f, "md"),
            Self::Json => write!(f, "json"),
        }
    }
}

impl std::str::FromStr for ExportFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "txt" => Ok(Self::Txt),
            "md" => Ok(Self::Md),
            "json" => Ok(Self::Json),
            _ => anyhow::bail!("不支持的导出格式: {}", s),
        }
    }
}

/// 导出项目数据结构
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportData {
    pub version: String,
    pub project: Project,
    pub volumes: Vec<Volume>,
    pub chapters: Vec<Chapter>,
    pub contents: Vec<ChapterContentExport>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChapterContentExport {
    pub chapter_id: String,
    pub content_json: String,
    pub content_text: String,
}

/// 导出项目到文件
pub async fn export_project(
    pool: &SqlitePool,
    project_id: &str,
    format: ExportFormat,
    path: &Path,
) -> Result<()> {
    // 1. 加载项目数据
    let project = sqlx::query_as::<_, Project>(
        "SELECT * FROM projects WHERE id = ?",
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;

    let volumes = sqlx::query_as::<_, Volume>(
        "SELECT * FROM volumes WHERE project_id = ? ORDER BY order_index",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    let chapters = sqlx::query_as::<_, Chapter>(
        "SELECT * FROM chapters WHERE project_id = ? ORDER BY volume_id, order_index",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    // 2. 加载所有章节内容
    let mut contents = Vec::new();
    for chapter in &chapters {
        let content = sqlx::query_as::<_, ChapterContentExport>(
            "SELECT chapter_id, content_json, content_text FROM chapter_contents WHERE chapter_id = ?",
        )
        .bind(&chapter.id)
        .fetch_optional(pool)
        .await?;

        if let Some(c) = content {
            contents.push(c);
        }
    }

    // 3. 构建导出数据
    let export_data = ExportData {
        version: "1.0".to_string(),
        project,
        volumes,
        chapters,
        contents,
    };

    // 4. 写入文件
    match format {
        ExportFormat::Json => {
            let json = serde_json::to_string_pretty(&export_data)?;
            tokio::fs::write(path, json).await?;
        }
        ExportFormat::Txt | ExportFormat::Md => {
            let mut text = String::new();

            // 标题
            text.push_str(&format!("# {}\n\n", export_data.project.title));

            if let Some(summary) = &export_data.project.summary {
                text.push_str(&format!("{}\n\n", summary));
            }

            // 按卷组织章节
            for volume in &export_data.volumes {
                text.push_str(&format!("## {}\n\n", volume.title));

                let vol_chapters: Vec<&Chapter> = export_data
                    .chapters
                    .iter()
                    .filter(|c| c.volume_id == volume.id)
                    .collect();

                for chapter in vol_chapters {
                    text.push_str(&format!("### {}\n\n", chapter.title));

                    // 查找内容
                    if let Some(content) = export_data.contents.iter().find(|c| c.chapter_id == chapter.id) {
                        if format == ExportFormat::Txt {
                            text.push_str(&content.content_text);
                        } else {
                            // Markdown: 尝试解析 JSON 并转换
                            text.push_str(&content.content_text);
                        }
                        text.push_str("\n\n");
                    }
                }
            }

            tokio::fs::write(path, text).await?;
        }
    }

    Ok(())
}

/// 从 JSON 文件导入项目
pub async fn import_project_json(
    pool: &SqlitePool,
    path: &Path,
) -> Result<String> {
    // 1. 读取文件
    let json = tokio::fs::read_to_string(path).await?;
    let export_data: ExportData = serde_json::from_str(&json)?;

    // 2. 创建新项目
    let project_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    sqlx::query(
        "INSERT INTO projects (id, title, summary, genre, target_words, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&project_id)
    .bind(&export_data.project.title)
    .bind(&export_data.project.summary)
    .bind(&export_data.project.genre)
    .bind(export_data.project.target_words)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    // 3. 创建卷和章节
    for volume in &export_data.volumes {
        let vol_id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO volumes (id, project_id, title, order_index, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&vol_id)
        .bind(&project_id)
        .bind(&volume.title)
        .bind(volume.order_index)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await?;

        let vol_chapters: Vec<&Chapter> = export_data
            .chapters
            .iter()
            .filter(|c| c.volume_id == volume.id)
            .collect();

        for chapter in vol_chapters {
            let ch_id = uuid::Uuid::new_v4().to_string();

            sqlx::query(
                "INSERT INTO chapters (id, volume_id, project_id, title, status, word_count, order_index, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&ch_id)
            .bind(&vol_id)
            .bind(&project_id)
            .bind(&chapter.title)
            .bind(&chapter.status)
            .bind(chapter.word_count)
            .bind(chapter.order_index)
            .bind(&now)
            .bind(&now)
            .execute(pool)
            .await?;

            // 插入内容
            if let Some(content) = export_data.contents.iter().find(|c| c.chapter_id == chapter.id) {
                sqlx::query(
                    "INSERT INTO chapter_contents (chapter_id, content_json, content_text, updated_at)
                     VALUES (?, ?, ?, ?)",
                )
                .bind(&ch_id)
                .bind(&content.content_json)
                .bind(&content.content_text)
                .bind(&now)
                .execute(pool)
                .await?;
            }
        }
    }

    Ok(project_id)
}
