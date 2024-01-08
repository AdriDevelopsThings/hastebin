use axum::response::Html;

pub async fn index_html() -> Html<&'static str> {
    Html(include_str!("public/index.html"))
}
