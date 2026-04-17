mod checkers;
mod config;
mod db;
mod error;
mod models;
mod routes;
mod scheduler;
mod state;
mod ws;

use state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cfg = config::Config::from_env();

    let pool = db::create_pool(&cfg.database_url).await;
    db::run_migrations(&pool).await;

    let app_state = AppState::new(pool.clone());

    scheduler::start_all(&pool, app_state.tx.clone(), &app_state.scheduler_handles).await;

    // Heartbeat task: ping all WS clients every 30s
    let ping_tx = app_state.tx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            let ts = chrono::Utc::now().to_rfc3339();
            let _ = ping_tx.send(ws::messages::WsMessage::Ping { ts });
        }
    });

    let router = routes::router(app_state);
    let addr = format!("0.0.0.0:{}", cfg.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Listening on {}", addr);
    axum::serve(listener, router).await.unwrap();
}
