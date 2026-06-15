use axum::{
    routing::{delete, get, post},
    Json, Router,
};
use axum_prometheus::PrometheusMetricLayerBuilder;
use dashmap::DashMap;
use sqlx::PgPool;
use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};
use tokio::sync::watch;
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    OpenApi,
};
use utoipa_scalar::{Scalar, Servable};
use uuid::Uuid;

pub mod auth;
pub mod grades;
pub mod internal;

#[derive(OpenApi)]
#[openapi(
      paths(
        // Auth paths
        crate::routes::auth::foward_to_gradegetter,
        crate::routes::auth::forward_status_for_client,
        crate::routes::auth::add_schoology_credentials_handler,
        crate::routes::auth::delete_schoology_credentials_handler,
        crate::routes::auth::health,
        // Grade path
        crate::routes::grades::grades_handler,
        // Internal Paths
        crate::routes::internal::invalidate_user,
        crate::routes::internal::delete_handler,
        crate::routes::internal::forward_status_ws,
    ),
    components(
        schemas(
            backend_common::AuthenticatedUser,
            backend_common::Claims,
            backend_common::gradegetter::SchoologyLogin,
            backend_common::gradegetter::ForwardMessage,
        )
    ),
    modifiers(&JwtBearer, &InternalAuth),
    tags(
        (name = "user_auth", description = "Authentication endpoints"),
        (name = "grades", description = "Grade Endpoints"),
        (name = "internal", description = "Internal Endpoints")
    )
)]
pub struct DaApiDoc;

struct JwtBearer;
struct InternalAuth;

impl utoipa::Modify for JwtBearer {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    ),
                ),
            )
        }
    }
}

impl utoipa::Modify for InternalAuth {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "internal_auth",
                SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Basic).build()),
            );
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub seen_users: Arc<RwLock<HashSet<Uuid>>>,
    pub channels: Arc<DashMap<String, watch::Sender<String>>>,
}

pub fn create_routes(pool: PgPool) -> Router {
    let (prometheus_layer, metric_handle) = PrometheusMetricLayerBuilder::new()
        .with_prefix("gradegetter_backend")
        .with_default_metrics()
        .build_pair();

    let openapi = DaApiDoc::openapi();

    let seen_users = Arc::new(RwLock::new(HashSet::new()));
    let channels = Arc::new(DashMap::new());

    let app_state = AppState {
        pool,
        seen_users,
        channels,
    };

    let routes_without_middleware = Router::new()
        .route(
            "/auth/forward_ws/{uuid}",
            get(auth::forward_status_for_client),
        )
        .route("/health", get(auth::health))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        // Fowrard route here bc u cant add authorization headers to websockets in browser :(
        .route(
            "/api-docs/openapi.json",
            get({
                let json_spec = openapi.clone();
                move || async { Json(json_spec) }
            }),
        );

    let routes_with_middleware = Router::new()
        // Auth Routes
        .route("/auth/forward", get(auth::foward_to_gradegetter))
        .route(
            "/auth/schoology/credentials",
            post(auth::add_schoology_credentials_handler)
                .delete(auth::delete_schoology_credentials_handler),
        )
        // Grade Route
        .route("/grades", get(grades::grades_handler))
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            crate::middleware::jwt::jwt_auth,
        ));

    let internal_routes = Router::new()
        .route(
            "/internal/invalidate/{uuid}",
            get(internal::invalidate_user),
        )
        .route("/internal/delete/{uuid}", delete(internal::delete_handler))
        .route("/internal/forward_ws", get(internal::forward_status_ws))
        .layer(axum::middleware::from_fn(
            crate::middleware::internal::basic_auth,
        ));

    Router::new()
        .merge(routes_with_middleware)
        .merge(internal_routes)
        .merge(routes_without_middleware)
        .merge(Scalar::with_url("/scalar", openapi))
        .with_state(app_state)
        .layer(prometheus_layer)
}
