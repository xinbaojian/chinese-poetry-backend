use axum::{
    body::Body,
    extract::Request,
    http::{header, StatusCode},
    middleware,
    response::Response,
    routing::{delete, get, post, put},
    Router,
};
use rust_embed::Embed;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::handlers::admin;
use crate::handlers::api;
use crate::state::AppState;

#[derive(Embed, Clone)]
#[folder = "frontend/dist/"]
struct Assets;

/// 从 rust-embed 提供静态文件，未命中则回退 index.html（SPA 路由）
async fn serve_assets(req: Request) -> Response {
    let path = req.uri().path().trim_start_matches('/');

    // 尝试匹配精确路径
    if let Some(file) = Assets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime.as_ref())
            .body(Body::from(file.data.into_owned()))
            .unwrap();
    }

    // SPA fallback: 返回 index.html
    match Assets::get("index.html") {
        Some(file) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(Body::from(file.data.into_owned()))
            .unwrap(),
        None => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(Body::from(
                "OK — backend running.\nUse `pnpm dev` in frontend/ for development, or `pnpm build` for production.",
            ))
            .unwrap(),
    }
}

pub fn create_app(state: AppState) -> Router {
    // Admin API public routes (login + refresh)
    let admin_public = Router::new()
        .route("/api/v1/admin/login", post(admin::auth::login))
        .route("/api/v1/admin/refresh", post(admin::auth::refresh));

    // Admin API protected routes (JWT + role=admin)
    let admin_protected = Router::new()
        .route("/api/v1/admin/dashboard", get(admin::dashboard::dashboard))
        .route("/api/v1/admin/poets", get(admin::poets::list).post(admin::poets::create))
        .route("/api/v1/admin/poets/dynasties", get(admin::poets::get_dynasties))
        .route("/api/v1/admin/poets/:id", put(admin::poets::update).delete(admin::poets::delete))
        .route("/api/v1/admin/poems", get(admin::poems::list).post(admin::poems::create))
        .route("/api/v1/admin/poems/filter-options", get(admin::poems::get_filter_options))
        .route("/api/v1/admin/poems/poets", get(admin::poems::get_poets))
        .route("/api/v1/admin/poems/:id", get(admin::poems::get_poem).put(admin::poems::update).delete(admin::poems::delete))
        .route("/api/v1/admin/users", get(admin::users::list))
        .route("/api/v1/admin/users/:id", delete(admin::users::delete))
        .route("/api/v1/admin/users/:id/reset-password", put(admin::users::reset_password))
        .route("/api/v1/admin/users/:id/progress", get(admin::users::get_progress))
        .route("/api/v1/admin/import", post(admin::import::import_poems))
        .route("/api/v1/admin/export/users", get(admin::export::get_users))
        .route("/api/v1/admin/export/download", get(admin::export::download))
        .layer(middleware::from_fn_with_state(state.clone(), crate::auth::admin_api_auth_middleware));

    // App API public routes (register/login/refresh/logout)
    let api_public = Router::new()
        .route("/api/v1/auth/register", post(api::auth::register))
        .route("/api/v1/auth/login", post(api::auth::login))
        .route("/api/v1/auth/refresh", post(api::auth::refresh))
        .route("/api/v1/auth/logout", post(api::auth::logout));

    // App API protected routes (JWT auth)
    let api_protected = Router::new()
        .route("/api/v1/poems", get(api::poems::list_poems))
        .route("/api/v1/poems/:id", get(api::poems::get_poem))
        .route("/api/v1/progress", get(api::progress::get_progress).post(api::progress::sync_progress))
        .route("/api/v1/progress/due", get(api::progress::get_due_reviews))
        .route("/api/v1/progress/:poem_id", delete(api::progress::delete_progress))
        .route("/api/v1/auth/password", put(api::auth::change_password))
        .layer(middleware::from_fn_with_state(state.clone(), crate::auth::api_auth_middleware));

    Router::new()
        .merge(admin_public)
        .merge(admin_protected)
        .merge(api_public)
        .merge(api_protected)
        .fallback(serve_assets)
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
}
