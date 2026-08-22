use sqlx::SqlitePool;

use crate::db::repo::prompt_template_repo;
use crate::error::AppError;
use crate::models::prompt_template::{CreatePromptTemplate, PromptTemplate, UpdatePromptTemplate};

// 内置叙事规则（宽松/严格）的编译期回退默认值：
// 当数据库里找不到对应模板时用这些兜底，避免首次启动还没跑迁移时崩。
pub const DEFAULT_NARRATIVE_LOOSE: &str = "\
## 风格与质量控制
- 保持与前文一致的叙事视角和语言风格
- 对话自然，符合角色身份和性格
- 节奏张弛有度，避免平铺直叙
- 如果涉及设定，请贴合世界观，不要自创矛盾设定";

pub const DEFAULT_NARRATIVE_STRICT: &str = "\
## 叙事规则
- 视角：紧贴主角，不写他不知道的事
- 节奏：短句用于紧张处，长句后接短句，每段≤5句
- 情绪：用动作和生理反应，不用\"XX地\"或\"XX之色\"
- 禁用：如同、宛如、仿佛、正是、微微、轻轻、渐渐
- 禁止：开头写环境、人物直接报身份、结尾做总结/点题
- 对话：必须有功能，穿插动作，不同角色说话要有区分度";

const BUILTIN_LOOSE_ID: &str = "builtin-narrative-loose";
const BUILTIN_STRICT_ID: &str = "builtin-narrative-strict";

pub async fn list(pool: &SqlitePool) -> Result<Vec<PromptTemplate>, AppError> {
    prompt_template_repo::list(pool).await.map_err(AppError::from)
}

pub async fn list_by_type(
    pool: &SqlitePool,
    template_type: &str,
) -> Result<Vec<PromptTemplate>, AppError> {
    prompt_template_repo::list_by_type(pool, template_type)
        .await
        .map_err(AppError::from)
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<PromptTemplate, AppError> {
    prompt_template_repo::get(pool, id).await.map_err(AppError::from)
}

/// 按约束程度取对应的叙事规则正文：
/// - `Some("loose")` → narrative_loose；
/// - 其他 → narrative_strict；
/// 数据库里都找不到时回退到编译期默认值。
pub async fn get_narrative_rules(
    pool: &SqlitePool,
    constraint: Option<&str>,
) -> Result<String, AppError> {
    let template_type = match constraint {
        Some("loose") => "narrative_loose",
        _ => "narrative_strict",
    };

    if let Some(t) = prompt_template_repo::get_first_by_type(pool, template_type).await? {
        return Ok(t.content);
    }

    // 兜底：还没跑迁移/老库缺行时，按类型返回编译期默认值
    Ok(match template_type {
        "narrative_loose" => DEFAULT_NARRATIVE_LOOSE.to_string(),
        _ => DEFAULT_NARRATIVE_STRICT.to_string(),
    })
}

/// 启动时保证 narrative_loose / narrative_strict 两条内置模板存在：
/// 已有则跳过，缺失则以编译期默认内容创建（不会覆盖用户自定义修改）。
pub async fn ensure_narrative_builtins(pool: &SqlitePool) -> Result<(), AppError> {
    let loose_exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM prompt_templates WHERE id = ?",
    )
    .bind(BUILTIN_LOOSE_ID)
    .fetch_one(pool)
    .await
    .unwrap_or(0) > 0;
    if !loose_exists {
        prompt_template_repo::create(
            pool,
            &CreatePromptTemplate {
                id: Some(BUILTIN_LOOSE_ID.to_string()),
                name: "叙事规则（宽松）".to_string(),
                template_type: "narrative_loose".to_string(),
                content: DEFAULT_NARRATIVE_LOOSE.to_string(),
                variables_json: Some("[]".to_string()),
                description: Some("宽松约束下的写作风格与质量控制规则".to_string()),
                is_builtin: Some(true),
            },
        )
        .await?;
    }

    let strict_exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM prompt_templates WHERE id = ?",
    )
    .bind(BUILTIN_STRICT_ID)
    .fetch_one(pool)
    .await
    .unwrap_or(0) > 0;
    if !strict_exists {
        prompt_template_repo::create(
            pool,
            &CreatePromptTemplate {
                id: Some(BUILTIN_STRICT_ID.to_string()),
                name: "叙事规则（严格）".to_string(),
                template_type: "narrative_strict".to_string(),
                content: DEFAULT_NARRATIVE_STRICT.to_string(),
                variables_json: Some("[]".to_string()),
                description: Some(
                    "严格约束下的叙事规则（节奏、视角、措辞、禁用词等）".to_string(),
                ),
                is_builtin: Some(true),
            },
        )
        .await?;
    }

    Ok(())
}

/// 按 id 把指定模板重置为编译期默认内容（内置模板一键「恢复默认」）。
/// 非内置 id 会返回 NotFound。
pub async fn reset_builtin(
    pool: &SqlitePool,
    id: &str,
) -> Result<PromptTemplate, AppError> {
    let (default_name, template_type, default_content, description) = match id {
        BUILTIN_LOOSE_ID => (
            "叙事规则（宽松）",
            "narrative_loose",
            DEFAULT_NARRATIVE_LOOSE,
            "宽松约束下的写作风格与质量控制规则",
        ),
        BUILTIN_STRICT_ID => (
            "叙事规则（严格）",
            "narrative_strict",
            DEFAULT_NARRATIVE_STRICT,
            "严格约束下的叙事规则（节奏、视角、措辞、禁用词等）",
        ),
        _ => {
            return Err(AppError::NotFound(format!(
                "Builtin prompt template {} not found or not resettable",
                id
            )));
        }
    };

    let updated = prompt_template_repo::upsert(
        pool,
        &CreatePromptTemplate {
            id: Some(id.to_string()),
            name: default_name.to_string(),
            template_type: template_type.to_string(),
            content: default_content.to_string(),
            variables_json: Some("[]".to_string()),
            description: Some(description.to_string()),
            is_builtin: Some(true),
        },
    )
    .await?;
    Ok(updated)
}

pub async fn create(
    pool: &SqlitePool,
    input: &CreatePromptTemplate,
) -> Result<PromptTemplate, AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Name must not be empty".into()));
    }
    if input.template_type.trim().is_empty() {
        return Err(AppError::Validation("Template type must not be empty".into()));
    }
    if input.content.trim().is_empty() {
        return Err(AppError::Validation("Content must not be empty".into()));
    }
    prompt_template_repo::create(pool, input).await.map_err(AppError::from)
}

pub async fn update(
    pool: &SqlitePool,
    id: &str,
    input: &UpdatePromptTemplate,
) -> Result<PromptTemplate, AppError> {
    prompt_template_repo::update(pool, id, input)
        .await
        .map_err(AppError::from)
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    prompt_template_repo::delete(pool, id).await.map_err(AppError::from)
}
