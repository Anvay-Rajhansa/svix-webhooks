// SPDX-FileCopyrightText: © 2022 Svix Authors
// SPDX-License-Identifier: MIT

#![warn(clippy::all)]
#![forbid(unsafe_code)]

use axum::{extract::Extension, Router};

use cfg::ConfigurationInner;
use lazy_static::lazy_static;
use opentelemetry::runtime::Tokio;
use opentelemetry_otlp::WithExportConfig;
use std::{
    net::TcpListener,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use tower::ServiceBuilder;
use tower_http::cors::{AllowHeaders, Any, CorsLayer};
use tracing_subscriber::{prelude::*, util::SubscriberInitExt};

use crate::{
    cfg::{CacheBackend, Configuration},
    core::{
        cache, idempotency::IdempotencyService, operational_webhooks::OperationalWebhookSenderInner,
    },
    db::init_db,
    expired_message_cleaner::expired_message_cleaner_loop,
    worker::worker_loop,
};

pub mod cfg;
pub mod core;
pub mod db;
pub mod error;
pub mod expired_message_cleaner;
pub mod queue;
pub mod redis;
pub mod v1;
pub mod worker;

const CRATE_NAME: &str = env!("CARGO_CRATE_NAME");

lazy_static! {
    pub static ref SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);
}

async fn graceful_shutdown_handler() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let sigterm = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let sigterm = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = sigterm => {},
    }

    tracing::info!("Received shutdown signal. Shutting down gracefully...");
    SHUTTING_DOWN.store(true, Ordering::SeqCst)
}

#[tracing::instrument(name = "app_start", level = "trace", skip_all)]
pub async fn run(cfg: Configuration, listener: Option<TcpListener>) {
    run_with_prefix(None, cfg, listener).await
}

// Made public for the purpose of E2E testing in which a queue prefix is necessary to avoid tests
// consuming from each others' queues
pub async fn run_with_prefix(
    prefix: Option<String>,
    cfg: Configuration,
    listener: Option<TcpListener>,
) {
    let pool = init_db(&cfg).await;

    tracing::debug!("Cache type: {:?}", cfg.cache_type);
    let cache = match cfg.cache_backend() {
        CacheBackend::None => cache::none::new(),
        CacheBackend::Memory => cache::memory::new(),
        CacheBackend::Redis(dsn) => {
            let mgr = crate::redis::new_redis_pool(dsn, &cfg).await;
            cache::redis::new(mgr)
        }
        CacheBackend::RedisCluster(dsn) => {
            let mgr = crate::redis::new_redis_pool_clustered(dsn, &cfg).await;
            cache::redis::new(mgr)
        }
    };

    tracing::debug!("Queue type: {:?}", cfg.queue_type);
    let (queue_tx, queue_rx) = queue::new_pair(&cfg, prefix.as_deref()).await;

    let op_webhook_sender = OperationalWebhookSenderInner::new(
        cfg.jwt_secret.clone(),
        cfg.operational_webhook_address.clone(),
    );

    let svc_cache = cache.clone();
    // build our application with a route
    let app = Router::new()
        .nest("/api/v1", v1::router())
        .merge(docs::router())
        .layer(
            ServiceBuilder::new().layer_fn(move |service| IdempotencyService {
                cache: svc_cache.clone(),
                service,
            }),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(AllowHeaders::mirror_request())
                .max_age(Duration::from_secs(600)),
        )
        .layer(Extension(pool.clone()))
        .layer(Extension(queue_tx.clone()))
        .layer(Extension(cfg.clone()))
        .layer(Extension(cache.clone()))
        .layer(Extension(op_webhook_sender.clone()));

    let with_api = cfg.api_enabled;
    let with_worker = cfg.worker_enabled;

    let listen_address = cfg.listen_address;

    let (server, worker_loop, expired_message_cleaner_loop) = tokio::join!(
        async {
            if with_api {
                if let Some(l) = listener {
                    tracing::debug!("API: Listening on {}", l.local_addr().unwrap());
                    axum::Server::from_tcp(l)
                        .expect("Error starting http server")
                        .serve(app.into_make_service())
                        .with_graceful_shutdown(graceful_shutdown_handler())
                        .await
                } else {
                    tracing::debug!("API: Listening on {}", listen_address);
                    axum::Server::bind(&listen_address)
                        .serve(app.into_make_service())
                        .with_graceful_shutdown(graceful_shutdown_handler())
                        .await
                }
            } else {
                tracing::debug!("API: off");
                graceful_shutdown_handler().await;
                Ok(())
            }
        },
        async {
            if with_worker {
                tracing::debug!("Worker: Initializing");
                worker_loop(
                    &cfg,
                    &pool,
                    cache.clone(),
                    queue_tx,
                    queue_rx,
                    op_webhook_sender,
                )
                .await
            } else {
                tracing::debug!("Worker: off");
                Ok(())
            }
        },
        async {
            if with_worker {
                tracing::debug!("Expired message cleaner: Initializing");
                expired_message_cleaner_loop(&pool).await
            } else {
                tracing::debug!("Expired message cleaner: off");
                Ok(())
            }
        }
    );

    server.expect("Error initializing server");
    worker_loop.expect("Error initializing worker");
    expired_message_cleaner_loop.expect("Error initializing expired message cleaner")
}

pub fn setup_tracing(cfg: &ConfigurationInner) {
    if std::env::var_os("RUST_LOG").is_none() {
        let level = cfg.log_level.to_string();
        let mut var = vec![
            format!("{CRATE_NAME}={level}"),
            format!("tower_http={level}"),
        ];

        if cfg.db_tracing {
            var.push(format!("sqlx={level}"));
        }

        std::env::set_var("RUST_LOG", var.join(","));
    }

    let otel_layer = cfg.opentelemetry_address.as_ref().map(|addr| {
        // Configure the OpenTelemetry tracing layer
        opentelemetry::global::set_text_map_propagator(
            opentelemetry::sdk::propagation::TraceContextPropagator::new(),
        );

        let exporter = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(addr);

        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(exporter)
            .with_trace_config(
                opentelemetry::sdk::trace::config()
                    .with_sampler(
                        cfg.opentelemetry_sample_ratio
                            .map(opentelemetry::sdk::trace::Sampler::TraceIdRatioBased)
                            .unwrap_or(opentelemetry::sdk::trace::Sampler::AlwaysOn),
                    )
                    .with_resource(opentelemetry::sdk::Resource::new(vec![
                        opentelemetry::KeyValue::new("service.name", "svix_server"),
                    ])),
            )
            .install_batch(Tokio)
            .unwrap();
        tracing_opentelemetry::layer().with_tracer(tracer)
    });

    // Then initialize logging with an additional layer priting to stdout. This additional layer is
    // either formatted normally or in JSON format
    // Fails if the subscriber was already initialized, which we can safely and silently ignore
    let _ = match cfg.log_format {
        cfg::LogFormat::Default => {
            let stdout_layer = tracing_subscriber::fmt::layer();
            tracing_subscriber::Registry::default()
                .with(otel_layer)
                .with(stdout_layer)
                .with(tracing_subscriber::EnvFilter::from_default_env())
                .try_init()
        }
        cfg::LogFormat::Json => {
            let fmt = tracing_subscriber::fmt::format().json().flatten_event(true);
            let json_fields = tracing_subscriber::fmt::format::JsonFields::new();

            let stdout_layer = tracing_subscriber::fmt::layer()
                .event_format(fmt)
                .fmt_fields(json_fields);

            tracing_subscriber::Registry::default()
                .with(otel_layer)
                .with(stdout_layer)
                .with(tracing_subscriber::EnvFilter::from_default_env())
                .try_init()
        }
    };
}

mod docs {
    use axum::{
        response::{Html, IntoResponse, Redirect},
        routing::get,
        Json, Router,
    };

    pub fn router() -> Router {
        Router::new()
            .route("/", get(|| async { Redirect::temporary("/docs") }))
            .route("/docs", get(get_docs))
            .route("/api/v1/openapi.json", get(get_openapi_json))
    }

    async fn get_docs() -> Html<&'static str> {
        Html(include_str!("static/docs.html"))
    }

    async fn get_openapi_json() -> impl IntoResponse {
        let json: serde_json::Value = serde_json::from_str(include_str!("static/openapi.json"))
            .expect("Error: openapi.json does not exist");
        Json(json)
    }
}
