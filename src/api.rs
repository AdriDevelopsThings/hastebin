use axum::{
    body::Bytes,
    extract,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::data_source::{DataSource, DataSourceError};

#[derive(Serialize)]
pub struct CreateFileResponse {
    id: String,
    change_key: String,
}

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

pub async fn create_file(
    extract::Path(filename): extract::Path<String>,
    extract::State(data_source): extract::State<DataSource>,
    body: Bytes,
) -> Result<Json<CreateFileResponse>, DataSourceError> {
    if body.len() as u64 > data_source.max_size {
        return Err(DataSourceError::TooBig);
    }
    let (id, change_key) = data_source.create_file(filename, body.to_vec()).await?;
    Ok(Json(CreateFileResponse { id, change_key }))
}

pub async fn modify_file(
    extract::Path((id, filename)): extract::Path<(String, String)>,
    extract::State(data_source): extract::State<DataSource>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, DataSourceError> {
    if body.len() as u64 > data_source.max_size {
        return Err(DataSourceError::TooBig);
    }
    if let Some(change_key) = headers.get("Change-Key") {
        let change_key = change_key.to_str().unwrap().to_string();
        data_source
            .modify_file(id, filename, change_key, body.to_vec())
            .await?;
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Ok((StatusCode::BAD_REQUEST, "Change key needed").into_response())
    }
}

pub async fn delete_file(
    extract::Path((id, filename)): extract::Path<(String, String)>,
    extract::State(data_source): extract::State<DataSource>,
    headers: HeaderMap,
) -> Result<Response, DataSourceError> {
    if let Some(change_key) = headers.get("Change-Key") {
        let change_key = change_key.to_str().unwrap().to_string();
        data_source.delete_file(id, filename, change_key).await?;
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Ok((StatusCode::BAD_REQUEST, "Change key needed").into_response())
    }
}
