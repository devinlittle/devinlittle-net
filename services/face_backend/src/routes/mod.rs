use std::sync::Arc;

use axum::{middleware, routing::get, Json, Router};
use axum_prometheus::PrometheusMetricLayerBuilder;
use hyper::StatusCode;
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    OpenApi,
};
use utoipa_scalar::{Scalar, Servable};

use crate::{
    middleware::{internal::basic_auth, jwt::jwt_auth},
    AppState,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::health,
    ),
    components(
        schemas(
        )
    ),
    modifiers(&JwtBearer, &InternalAuth),
    tags(
        (name = "internal", description = "internal routes"),
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

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "returns 200 if service alive"),
    ),
    tag = "internal"
)]
async fn health() -> StatusCode {
    StatusCode::OK
}

pub fn create_routes(app_state: Arc<AppState>) -> Router {
    let (prometheus_layer, metric_handle) = PrometheusMetricLayerBuilder::new()
        .with_prefix("nanopass_backend")
        .with_default_metrics()
        .build_pair();

    let openapi = DaApiDoc::openapi();

    let routes_without_middleware = Router::new()
        .route("/health", get(health))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .route(
            "/api-docs/openapi.json",
            get({
                let json_spec = openapi.clone();
                move || async { Json(json_spec) }
            }),
        );

    let routes_with_middleware =
        Router::new().layer(middleware::from_fn_with_state(app_state.clone(), jwt_auth));

    let internal_routes = Router::new().layer(middleware::from_fn(basic_auth));

    Router::new()
        .merge(routes_with_middleware)
        .merge(routes_without_middleware)
        .merge(internal_routes)
        .merge(Scalar::with_url("/scalar", openapi))
        .with_state(app_state)
        .layer(prometheus_layer)
}
