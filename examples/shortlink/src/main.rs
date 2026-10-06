use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".into());
    let db_path = env::var("SHORTLINK_DB").unwrap_or_else(|_| "shortlink.db".into());

    let app = shortlink::build_app(&db_path).await?;
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await?;
    println!("shortlink listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
