use axum::{
    http::header::CONTENT_TYPE,
    response::{Html, IntoResponse},
};

pub async fn index_html() -> Html<&'static str> {
    Html(include_str!("public/index.html"))
}

pub async fn index_js() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "text/javascript")],
        include_str!("public/index.js"),
    )
}
