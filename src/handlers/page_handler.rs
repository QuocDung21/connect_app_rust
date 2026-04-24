use axum::response::Html;

pub async fn index_handler() -> Html<String> {
    let html = tokio::fs::read_to_string("templates/index.html")
        .await
        .unwrap_or_else(|_| "Template not found".to_string());
    Html(html)
}
