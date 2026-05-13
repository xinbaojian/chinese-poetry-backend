mod app;
mod auth;
mod config;
mod errors;
mod handlers;
mod models;
mod state;

use std::path::Path;
use tracing_subscriber::prelude::*;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "chinese_poetry_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::load_config(Path::new("config.toml"))?;
    let pool = sqlx::MySqlPool::connect(&config.database.url).await?;

    // Run migrations manually
    run_migrations(&pool).await?;

    // Create default admin user if not exists
    seed_admin_user(&pool, &config).await?;

    let state = state::AppState {
        config: config.clone(),
        db: pool,
    };

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {}", addr);

    let app = app::create_app(state);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn run_migrations(pool: &sqlx::MySqlPool) -> anyhow::Result<()> {
    let migrations = vec![
        include_str!("../migrations/001_create_users.sql"),
        include_str!("../migrations/002_create_poets.sql"),
        include_str!("../migrations/003_create_poems.sql"),
        include_str!("../migrations/004_create_learning_records.sql"),
        include_str!("../migrations/005_create_review_history.sql"),
        include_str!("../migrations/006_create_settings.sql"),
    ];

    for sql in migrations {
        sqlx::query(sql).execute(pool).await?;
    }

    tracing::info!("Database migrations completed");
    Ok(())
}

async fn seed_admin_user(pool: &sqlx::MySqlPool, config: &config::Config) -> anyhow::Result<()> {
    let existing: Option<(u64,)> = sqlx::query_as(
        "SELECT id FROM users WHERE username = ?"
    )
    .bind(&config.auth.admin_username)
    .fetch_optional(pool)
    .await?;

    if existing.is_none() {
        let password = config.auth.admin_password.clone();
        let hash = tokio::task::spawn_blocking(move || {
            auth::hash_password(&password)
        }).await.map_err(|e| anyhow::anyhow!(e))?.map_err(|e| anyhow::anyhow!(e.to_string()))?;

        sqlx::query(
            "INSERT INTO users (username, password_hash, role) VALUES (?, ?, 'admin')"
        )
        .bind(&config.auth.admin_username)
        .bind(&hash)
        .execute(pool)
        .await?;

        tracing::info!("Admin user created: {}", config.auth.admin_username);
    }

    Ok(())
}
