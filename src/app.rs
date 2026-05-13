use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::handlers::admin;
use crate::handlers::api;
use crate::state::AppState;

const DIST_DIR: &str = "frontend/dist";

pub fn create_app(state: AppState) -> Router {
    // Admin API public routes (login only)
    let admin_public = Router::new()
        .route("/api/v1/admin/login", post(admin::auth::login));

    // Admin API protected routes (JWT + role=admin)
    let admin_protected = Router::new()
        .route("/api/v1/admin/dashboard", get(admin::dashboard::dashboard))
        .route("/api/v1/admin/poets", get(admin::poets::list).post(admin::poets::create))
        .route("/api/v1/admin/poets/dynasties", get(admin::poets::get_dynasties))
        .route("/api/v1/admin/poets/{id}", put(admin::poets::update).delete(admin::poets::delete))
        .route("/api/v1/admin/poems", get(admin::poems::list).post(admin::poems::create))
        .route("/api/v1/admin/poems/filter-options", get(admin::poems::get_filter_options))
        .route("/api/v1/admin/poems/poets", get(admin::poems::get_poets))
        .route("/api/v1/admin/poems/{id}", get(admin::poems::get_poem).put(admin::poems::update).delete(admin::poems::delete))
        .route("/api/v1/admin/users", get(admin::users::list))
        .route("/api/v1/admin/users/{id}", delete(admin::users::delete))
        .route("/api/v1/admin/import", post(admin::import::import_poems))
        .route("/api/v1/admin/export/users", get(admin::export::get_users))
        .route("/api/v1/admin/export/download", get(admin::export::download))
        .layer(middleware::from_fn_with_state(state.clone(), crate::auth::admin_api_auth_middleware));

    // App API public routes (register/login)
    let api_public = Router::new()
        .route("/api/v1/auth/register", post(api::auth::register))
        .route("/api/v1/auth/login", post(api::auth::login));

    // App API protected routes (JWT auth)
    let api_protected = Router::new()
        .route("/api/v1/poems", get(api::poems::list_poems))
        .route("/api/v1/poems/{id}", get(api::poems::get_poem))
        .route("/api/v1/progress", get(api::progress::get_progress).post(api::progress::sync_progress))
        .route("/api/v1/progress/due", get(api::progress::get_due_reviews))
        .layer(middleware::from_fn_with_state(state.clone(), crate::auth::api_auth_middleware));

    // SPA fallback: serves index.html for any non-API route
    async fn spa_index(_req: Request<Body>) -> Response {
        match tokio::fs::read_to_string(format!("{DIST_DIR}/index.html")).await {
            Ok(html) => Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/html; charset=utf-8")
                .body(Body::from(html))
                .unwrap(),
            Err(_) => Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/plain; charset=utf-8")
                .body(Body::from(
                    "OK — backend running.\nUse `pnpm dev` in frontend/ for development, or `pnpm build` for production.",
                ))
                .unwrap(),
        }
    }

    Router::new()
        .merge(admin_public)
        .merge(admin_protected)
        .merge(api_public)
        .merge(api_protected)
        .nest_service("/assets", ServeDir::new(format!("{DIST_DIR}/assets")))
        .route("/vite.svg", get(|| async {
            match tokio::fs::read(format!("{DIST_DIR}/vite.svg")).await {
                Ok(data) => Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "image/svg+xml")
                    .body(Body::from(data))
                    .unwrap(),
                Err(_) => StatusCode::NOT_FOUND.into_response(),
            }
        }))
        .fallback(spa_index)
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
}
