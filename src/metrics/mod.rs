// Сode in this file is poorly written, but it works and solves my needs.
// So let it be - but don't repeat after me. Maybe in future I'll improve that. Maybe.

use axum::{Router, body::Body, http::Request, routing::get};
use axum_prometheus::PrometheusMetricLayer;
use prometheus_client::{metrics::counter::Counter, registry::Registry};
use std::sync::{LazyLock, OnceLock};

use crate::config::env::BOT_CONFIG;

static REGISTRY: OnceLock<Registry> = OnceLock::new();

// Export special preconstructed counters for Teloxide's handlers.
pub static INLINE_COUNTER: LazyLock<Counter<u64>> =
    LazyLock::new(Counter::default);

pub static CALLBACK_COUNTER: LazyLock<Counter<u64>> =
    LazyLock::new(Counter::default);

pub static MESSAGE_COUNTER: LazyLock<Counter<u64>> =
    LazyLock::new(Counter::default);

pub static MESSAGE_HANDLED_COUNTER: LazyLock<Counter<u64>> =
    LazyLock::new(Counter::default);

pub static CMD_START_COUNTER: LazyLock<Counter<u64>> =
    LazyLock::new(Counter::default);

pub static CMD_HELP_COUNTER: LazyLock<Counter<u64>> =
    LazyLock::new(Counter::default);

pub static CMD_COUNTER: LazyLock<Counter<u64>> =
    LazyLock::new(Counter::default);

pub static UNHANDLED_COUNTER: LazyLock<Counter<u64>> =
    LazyLock::new(Counter::default);

pub static DUEL_NUMBERS: LazyLock<Counter<u64>> =
    LazyLock::new(Counter::default);

pub fn init() -> axum::Router {
    let mut prometheus = Registry::default();

    prometheus.register(
        "inline_usage",
        "count of inline queries processed by the bot",
        INLINE_COUNTER.clone(),
    );
    prometheus.register(
        "callback",
        "count of callbacks",
        CALLBACK_COUNTER.clone(),
    );
    prometheus.register(
        "message_usage",
        "count of messages processed",
        MESSAGE_COUNTER.clone(),
    );

    prometheus.register(
        "message_handled",
        "count of messages handled",
        MESSAGE_HANDLED_COUNTER.clone(),
    );

    prometheus.register(
        "command_start_usage",
        "count of /start invocations",
        CMD_START_COUNTER.clone(),
    );

    prometheus.register(
        "command_help_usage",
        "count of /help invocations",
        CMD_HELP_COUNTER.clone(),
    );

    prometheus.register(
        "command_all_usage",
        "count of commands invocations",
        CMD_COUNTER.clone(),
    );

    prometheus.register(
        "unhandled",
        "count of unhandled updates",
        UNHANDLED_COUNTER.clone(),
    );

    prometheus.register(
        "duel_numbers",
        "Active duels on time",
        DUEL_NUMBERS.clone(),
    );

    REGISTRY.set(prometheus).unwrap();

    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    let metrics_endpoint = |req: Request<Body>| async move {
        let headers = req.headers();
        if let Some(auth) =
            headers.get("Authorization").and_then(|v| v.to_str().ok())
        {
            if auth.len() > 7 && auth[7..] == BOT_CONFIG.prometheus_token {
                log::info!("Metrics: captured data");
                let mut buf = String::new();

                match prometheus_client::encoding::text::encode(
                    &mut buf,
                    REGISTRY.get().unwrap(),
                ) {
                    Ok(_) => {},
                    Err(_) => log::error!("Metrics: encoding error"),
                };

                buf.push_str(&metric_handle.render());

                return Ok(buf);
            }
        }

        log::warn!("Metrics: unauthorized");

        Err(axum::http::StatusCode::UNAUTHORIZED)
    };

    Router::new()
        .route("/metrics", get(metrics_endpoint))
        .layer(prometheus_layer)
}
