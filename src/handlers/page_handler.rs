use axum::response::Html;

pub async fn index_handler() -> Html<String> {
    let html = include_str!("../../templates/index.html").to_string();
    Html(html)
}
