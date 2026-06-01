use axum::response::Html;
use axum::extract::State;
use std::fs;
use std::sync::Arc;

use crate::AppState;

pub async fn index_handler(State(state): State<Arc<AppState>>) -> Html<String> {
    let mut html = fs::read_to_string("templates/index.html").unwrap_or_else(|_| include_str!("../../templates/index.html").to_string());
    html = html.replace("{{APP_PIN}}", &state.pin);
    Html(html)
}
