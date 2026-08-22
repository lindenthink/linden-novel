use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::Path;

/// 当前 UTC 时间，格式 `YYYY-MM-DDTHH:MM:SSZ`（ISO 8601 带 Z 后缀）
///
/// 统一用 UTC + Z 后缀，前端 `new Date(iso)` 能正确转本地时区显示。
pub fn now() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}


/// 通过 `sqlite3_auto_extension` 让每个新连接自动加载 vec0 模块。
/// 仅需在进程启动时调用一次。
fn register_sqlite_vec() {
    // 防止重复注册
    static REGISTERED: std::sync::Once = std::sync::Once::new();
    REGISTERED.call_once(|| unsafe {
        // 先转 usize 再 transmute，避免 ZST transmute 错误
        let fn_addr = sqlite_vec::sqlite3_vec_init as *const () as usize;
        let rc = libsqlite3_sys::sqlite3_auto_extension(Some(std::mem::transmute::<
            usize,
            unsafe extern "C" fn(
                *mut libsqlite3_sys::sqlite3,
                *mut *mut std::os::raw::c_char,
                *const libsqlite3_sys::sqlite3_api_routines,
            ) -> std::os::raw::c_int,
        >(fn_addr)));
        if rc == 0 {
            tracing::info!("sqlite-vec auto-extension registered");
        } else {
            tracing::warn!("sqlite-vec auto-extension registration failed: rc={}", rc);
        }
    });
}

/// 获取嵌入向量维度（通过环境变量配置，默认 512 适配 bge-small-zh-v1.5）
fn embed_dim() -> usize {
    std::env::var("LINDEN_EMBED_DIM")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(512)
}

/// 初始化 SQLite 连接池并执行迁移
///
/// sqlite-vec 扩展通过静态链接自动加载，vec0 虚拟表按配置维度创建。
/// 若扩展不可用或维度不匹配，检索自动回退到内存余弦搜索。
pub async fn init_pool(db_dir: &Path) -> Result<SqlitePool, sqlx::Error> {
    // 注册 sqlite-vec 扩展（必须在创建连接前完成）
    register_sqlite_vec();

    std::fs::create_dir_all(db_dir).ok();
    let db_path = db_dir.join("linden.db");

    let options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    // 运行迁移
    sqlx::migrate!().run(&pool).await?;

    // 创建 vec0 虚拟表（摘要级 + 切片级）
    try_create_vec0_tables(&pool).await;

    Ok(pool)
}

/// 尝试创建 vec0 虚拟表（摘要级 + 切片级）
///
/// project_id 用 `partition key` 关键字声明，KNN 查询时可直接 `WHERE project_id = ?`
/// 过滤，性能远优于全表扫描 + Rust 层后过滤。
async fn try_create_vec0_tables(pool: &SqlitePool) {
    let dim = embed_dim();

    match sqlx::query(&format!(
        "CREATE VIRTUAL TABLE IF NOT EXISTS embeddings_vec USING vec0(
            project_id   TEXT partition key,
            source_type  TEXT,
            source_id    TEXT,
            embedding    float[{}] distance_metric=cosine
        )",
        dim
    ))
    .execute(pool)
    .await
    {
        Ok(_) => tracing::info!("Created vec0 table: embeddings_vec (dim={})", dim),
        Err(e) => tracing::warn!("Skip embeddings_vec: {}", e),
    }

    match sqlx::query(&format!(
        "CREATE VIRTUAL TABLE IF NOT EXISTS chunks_vec USING vec0(
            project_id   TEXT partition key,
            chapter_id   TEXT,
            chunk_index  INTEGER,
            embedding    float[{}] distance_metric=cosine
        )",
        dim
    ))
    .execute(pool)
    .await
    {
        Ok(_) => tracing::info!("Created vec0 table: chunks_vec (dim={})", dim),
        Err(e) => tracing::warn!("Skip chunks_vec: {}", e),
    }
}
