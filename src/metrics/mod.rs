// Сode in this file is poorly written, but it works and solves my needs.
// So let it be - but don't repeat after me. Maybe in future I'll improve that. Maybe.

use axum::{Router, body::Body, http::Request, routing::get};
use axum_prometheus::PrometheusMetricLayer;
use prometheus::{Counter, Gauge, Registry, TextEncoder};
use std::sync::LazyLock;
use systemstat::{Platform, System};
use tokio::time::{Duration, sleep};

use crate::config::env::BOT_CONFIG;

// Export special preconstructed counters for Teloxide's handlers.
pub static INLINE_COUNTER: LazyLock<Counter> = LazyLock::new(|| {
    Counter::new(
        "inline_usage_total",
        "count of inline queries processed by the bot",
    )
    .unwrap()
});

pub static CALLBACK_COUNTER: LazyLock<Counter> = LazyLock::new(|| {
    Counter::new("callback_total", "count of callbacks").unwrap()
});

pub static MESSAGE_COUNTER: LazyLock<Counter> = LazyLock::new(|| {
    Counter::new(
        "message_usage_total",
        "count of messages processed by the bot",
    )
    .unwrap()
});

pub static MESSAGE_HANDLED_COUNTER: LazyLock<Counter> = LazyLock::new(|| {
    Counter::new(
        "message_handled_total",
        "count of messages handled by the bot",
    )
    .unwrap()
});

pub static CMD_START_COUNTER: LazyLock<Counter> = LazyLock::new(|| {
    Counter::new("command_start_usage_total", "count of /start invocations")
        .unwrap()
});

pub static CMD_HELP_COUNTER: LazyLock<Counter> = LazyLock::new(|| {
    Counter::new("command_help_usage_total", "count of /help invocations")
        .unwrap()
});

pub static CMD_COUNTER: LazyLock<Counter> = LazyLock::new(|| {
    Counter::new("command_all_usage", "count of commands invocations").unwrap()
});

pub static UNHANDLED_COUNTER: LazyLock<Counter> = LazyLock::new(|| {
    Counter::new("unhandled", "count of unhandled updates").unwrap()
});

pub static DUEL_NUMBERS: LazyLock<Counter> = LazyLock::new(|| {
    Counter::new("duel_numbers", "Active duels on time").unwrap()
});

static CPU_USAGE: LazyLock<Gauge> = LazyLock::new(|| {
    Gauge::new("cpu_usage", "Current CPU usage in percent").unwrap()
});

static MEM_USAGE: LazyLock<Gauge> = LazyLock::new(|| {
    Gauge::new("mem_usage", "Current memory usage in percent").unwrap()
});

pub fn init() -> axum::Router {
    let prometheus = Registry::new();

    let err = "Unable to register counter";

    prometheus.register(Box::new(INLINE_COUNTER.clone())).expect(err);
    prometheus.register(Box::new(CALLBACK_COUNTER.clone())).expect(err);
    prometheus.register(Box::new(MESSAGE_COUNTER.clone())).expect(err);
    prometheus.register(Box::new(MESSAGE_HANDLED_COUNTER.clone())).expect(err);
    prometheus.register(Box::new(UNHANDLED_COUNTER.clone())).expect(err);
    prometheus.register(Box::new(CMD_START_COUNTER.clone())).expect(err);
    prometheus.register(Box::new(CMD_HELP_COUNTER.clone())).expect(err);
    prometheus.register(Box::new(CMD_COUNTER.clone())).expect(err);
    prometheus.register(Box::new(DUEL_NUMBERS.clone())).expect(err);
    prometheus.register(Box::new(CPU_USAGE.clone())).expect(err);
    prometheus.register(Box::new(MEM_USAGE.clone())).expect(err);

    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    init_interval_listener();

    let metrics_endpoint = |req: Request<Body>| async move {
        let headers = req.headers();
        if let Some(auth) =
            headers.get("Authorization").and_then(|v| v.to_str().ok())
        {
            if auth.len() > 7 && auth[7..] == BOT_CONFIG.prometheus_token {
                log::info!("Metrics: captured data");
                let metrics = prometheus.gather();
                let mut buf = metric_handle.render();

                TextEncoder::new().encode_utf8(&metrics, &mut buf).unwrap();

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

                    CPU_USAGE.set(percentage.trunc());
                },
                Err(x) => crate::myerr!("CPU load: error: {x}"),
            }

            match sys.memory() {
                Ok(mem) => {
                    let mem_used = mem.total.0 - mem.free.0;
                    let percentage =
                        (mem_used as f64 / mem.total.0 as f64) * 100.0;

                    MEM_USAGE.set(percentage.trunc());
                },
                Err(x) => crate::myerr!("Memory: error: {x}"),
            }
            sleep(Duration::from_secs(5)).await;
        }
    });
}
