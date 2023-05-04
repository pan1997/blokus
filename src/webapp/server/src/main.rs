mod game;
mod account;

use axum::{
  http::{StatusCode, Uri},
  response::IntoResponse,
  routing::get,
  Router, Server,
};

pub async fn fallback(uri: Uri) -> impl IntoResponse {
  (StatusCode::NOT_FOUND, format!("No route {}", uri))
}

#[tokio::main]
async fn main() {
  let app = Router::new()
    .fallback(fallback)
    .route("/", get(|| async { "Hello axum!" }));

  Server::bind(&"0.0.0.0:3000".parse().unwrap())
    .serve(app.into_make_service())
    .await
    .unwrap();
}
