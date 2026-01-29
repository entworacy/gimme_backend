use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
};

#[derive(Template)]
#[template(path = "error/404.html")]
pub struct NotFoundTemplate;

#[derive(Template)]
#[template(path = "error/500.html")]
pub struct InternalErrorTemplate;

/// Handler for 404 Not Found
pub async fn handler_404() -> impl IntoResponse {
    let template = NotFoundTemplate;
    match template.render() {
        Ok(html) => (StatusCode::NOT_FOUND, Html(html)).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error (Template Rendering Failed)",
        )
            .into_response(),
    }
}

/// Handler for 500 Internal Server Error (to be used in CatchPanicLayer)
pub fn handler_500(_err: Box<dyn std::any::Any + Send + 'static>) -> axum::response::Response {
    let template = InternalErrorTemplate;
    match template.render() {
        Ok(html) => (StatusCode::INTERNAL_SERVER_ERROR, Html(html)).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error (Template Rendering Failed)",
        )
            .into_response(),
    }
}
