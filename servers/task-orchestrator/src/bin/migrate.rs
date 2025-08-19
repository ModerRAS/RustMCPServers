use sqlx::{migrate::MigrateDatabase, Sqlite, Pool};
use std::env;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 从环境变量获取数据库URL
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:///data/tasks.db".to_string());

    println!("Running database migrations for: {}", database_url);

    // 创建数据库连接池
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?;

    // 检查数据库是否需要迁移
    if !Sqlite::database_exists(&database_url).await? {
        println!("Database does not exist, creating...");
        Sqlite::create_database(&database_url).await?;
    }

    // 运行迁移
    println!("Running migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;

    println!("Database migrations completed successfully!");

    Ok(())
}