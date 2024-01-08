use std::env;

use axum::{routing, Router};
use log::info;
use tokio::net::TcpListener;

use crate::{
    api::{create_file, delete_file, get_file, modify_file},
    data_source::DataSource,
};

mod api;
mod auto_delete;
mod data_source;
mod id;
mod static_files;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let data_source = DataSource::from_env();
    data_source.create_directory().await;
    data_source.clone().start_auto_delete();
    let app = Router::new()
        .route("/", routing::get(static_files::index_html))
        .route("/:id/:filename", routing::get(static_files::index_html))
        .route("/index.js", routing::get(static_files::index_js))
        .route("/api/file/:id/:filename", routing::get(get_file))
        .route("/api/file/:id/:filename", routing::put(modify_file))
        .route("/api/file/:id/:filename", routing::delete(delete_file))
        .route("/api/file/:filename", routing::post(create_file))
        .with_state(data_source);
    let listen_address =
        env::var("LISTEN_ADDRESS").unwrap_or_else(|_| String::from("127.0.0.1:8000"));
    let listener = TcpListener::bind(&listen_address)
        .await
        .expect("Error while listening");
    info!("HTTP server listening on http://{listen_address}");
    axum::serve(listener, app)
        .await
        .expect("Error while serving");
}
