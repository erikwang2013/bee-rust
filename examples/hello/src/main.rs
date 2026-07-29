// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
#[tokio::main]
async fn main() {
    let _log_handle = bee_rust::bee_logs::Logger::new().init().unwrap();
    let router = bee_rust::bee_router::Router::new().ns("/api/v1", |ns| ns);
    tracing::info!("Starting bee-rust on http://localhost:8080");
    let app = router.build();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
