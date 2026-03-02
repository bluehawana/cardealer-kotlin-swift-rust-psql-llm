use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

/// Create a PostgreSQL connection pool and run embedded migrations.
pub async fn connect(url: &str, max_conns: u32) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(max_conns)
        .connect(url)
        .await?;

    tracing::info!("connected to PostgreSQL");

    // Run embedded migrations
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("database migrations applied");

    Ok(pool)
}
