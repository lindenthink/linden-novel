use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::Path;

/// 当前本地时间，格式 `YYYY-MM-DD HH:MM:SS`
pub fn now() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 初始化 SQLite 连接池并执行迁移
pub async fn init_pool(db_dir: &Path) -> Result<SqlitePool, sqlx::Error> {
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

    sqlx::migrate!().run(&pool).await?;
    Ok(pool)
}
