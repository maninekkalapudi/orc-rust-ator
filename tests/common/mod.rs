// In tests/common/mod.rs

use anyhow::Result;
use orc_rust_ator::orchestrator::scheduler::Scheduler;
use orc_rust_ator::orchestrator::worker_manager::WorkerManager;
use orc_rust_ator::state::db::Db;
use std::env;

pub async fn setup() -> Result<String> {
    // Initialize the global logger.
    let _ = orc_rust_ator::logger::initialize_logger();

    // 1. Set the database URL to an in-memory SQLite database for testing.
    let database_url = "sqlite::memory:";
    env::set_var("TEST_DATABASE_URL", database_url);

    // 2. Create a new Db instance.
    let db = Db::new(database_url).await?;

    // 3. Run the database migrations.
    db.migrate().await?;

    // 4. Launch the Scheduler and WorkerManager in the background.
    let scheduler = Scheduler::new(db.clone());
    let worker_manager = WorkerManager::new(db.clone());

    tokio::spawn(async move {
        scheduler.run().await.unwrap();
    });

    tokio::spawn(async move {
        worker_manager.run().await.unwrap();
    });

    // 5. Start the axum server in the background.
    let app = orc_rust_ator::api::app(db);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let server_url = format!("http://{}", addr);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    Ok(server_url)
}
