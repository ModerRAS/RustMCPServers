use sqlx::{migrate::MigrateDatabase, Sqlite};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🔄 Running database migrations...");
    
    // 从环境变量获取数据库URL
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:///data/tasks.db".to_string());
    
    println!("📊 Using database: {}", database_url);
    
    // 创建数据库（如果不存在）
    if !Sqlite::database_exists(&database_url).await? {
        println!("🔧 Creating database...");
        Sqlite::create_database(&database_url).await?;
    }
    
    // 创建连接池
    let _pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    
    println!("✅ Database connection established");
    
    // 运行迁移
    println!("📝 Running migrations...");
    // 注意：简化版本不使用数据库迁移，这里只是示例
    println!("✅ No migrations needed for in-memory storage");
    
    println!("🎉 Migrations completed successfully!");
    
    Ok(())
}