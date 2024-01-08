use std::env;

use axum::{routing, Router};
use tokio::net::TcpListener;

use crate::{api::get_file, data_source::DataSource};

mod api;
mod data_source;
mod static_files;

#[tokio::main]
async fn main() {
    let data_source = DataSource::from_env();
    data_source.create_directory().await;
    let app = Router::new()
        .route("/", routing::get(static_files::index_html))
        .route("/api/file/:id/:filename", routing::get(get_file))
        .with_state(data_source);
    let listen_address =
        env::var("LISTEN_ADDRESS").unwrap_or_else(|_| String::from("127.0.0.1:8000"));
    let listener = TcpListener::bind(&listen_address)
        .await
        .expect("Error while listening");
    println!("HTTP server listening on http://{listen_address}");
    axum::serve(listener, app)
        .await
        .expect("Error while serving");
}
