// Сode in this file is poorly written, but it works and solves my needs.
// So let it be - but don't repeat after me. Maybe in future I'll improve that. Maybe.

use axum::{Router, body::Body, http::Request, routing::get};
use axum_prometheus::PrometheusMetricLayer;
use prometheus_client::{
    metrics::{counter::Counter, gauge::Gauge},
    registry::Registry,
};
use std::sync::{LazyLock, OnceLock, atomic::AtomicI64};
use systemstat::{Platform, System};
use tokio::time::{Duration, sleep};

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

static CPU_USAGE: LazyLock<Gauge<i64, AtomicI64>> =
    LazyLock::new(Gauge::default);

static MEM_USAGE: LazyLock<Gauge<i64, AtomicI64>> =
    LazyLock::new(Gauge::default);

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

    prometheus.register(
        "cpu_usage",
        "Current CPU usage in percent",
        CPU_USAGE.clone(),
    );

    prometheus.register(
        "mem_usage",
        "Current memory usage in percent",
        MEM_USAGE.clone(),
    );

    REGISTRY.set(prometheus).unwrap();

    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    init_interval_listener();

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

fn init_interval_listener() {
    let sys = System::new();

    tokio::spawn(async move {
        sleep(Duration::from_secs(1)).await;
        loop {
            match sys.cpu_load_aggregate() {
                Ok(cpu) => {
                    sleep(Duration::from_secs(1)).await;

                    let cpu = cpu.done().expect("CPU metrics crash!");
                    let percentage = (cpu.system + cpu.user) as f64 * 100.0;

                    CPU_USAGE.set(percentage.trunc() as i64);
                },
                Err(x) => crate::myerr!("CPU load: error: {x}"),
            }

            match sys.memory() {
                Ok(mem) => {
                    let mem_used = mem.total.0 - mem.free.0;
                    let percentage =
                        (mem_used as f64 / mem.total.0 as f64) * 100.0;

                    MEM_USAGE.set(percentage.trunc() as i64);
                },
                Err(x) => crate::myerr!("Memory: error: {x}"),
            }
            sleep(Duration::from_secs(5)).await;
        }
    });
}
