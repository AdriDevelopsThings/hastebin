use axum::{
    extract,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};

use crate::data_source::DataSource;

pub async fn get_file(
    extract::Path((id, filename)): extract::Path<(String, String)>,
    extract::State(data_source): extract::State<DataSource>,
) -> Response {
    if let Some(file) = data_source.get_file(id, filename.clone()).await {
        let mime = mime_guess::from_path(filename).first_or_text_plain();
        ([(header::CONTENT_TYPE, mime.to_string())], file).into_response()
    } else {
        (StatusCode::NOT_FOUND, "Not Found").into_response()
    }
}
