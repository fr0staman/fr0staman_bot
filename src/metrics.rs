// Сode in this file is poorly written, but it works and solves my needs.
// So let it be - but don't repeat after me. Maybe in future I'll improve that. Maybe.

use axum::{body::Body, http::Request, routing::get};
use axum_prometheus::PrometheusMetricLayer;
use once_cell::sync::Lazy;
use prometheus::{Encoder, Gauge, Opts, TextEncoder};
use systemstat::{Platform, System};
use tokio::time::{sleep, Duration};

use crate::config::BOT_CONFIG;
// Register additional metrics of our own structs by using this registry instance.
pub static REGISTRY: Lazy<Registry> =
    Lazy::new(|| Registry(prometheus::Registry::new()));

// Export special preconstructed counters for Teloxide's handlers.
pub static INLINE_COUNTER: Lazy<Counter> = Lazy::new(|| {
    Counter::new(
        "inline",
        Opts::new(
            "inline_usage_total",
            "count of inline queries processed by the bot",
        ),
    )
});

pub static CALLBACK_COUNTER: Lazy<Counter> = Lazy::new(|| {
    Counter::new("callback", Opts::new("callback_total", "count of callbacks"))
});

pub static MESSAGE_COUNTER: Lazy<Counter> = Lazy::new(|| {
    Counter::new(
        "message",
        Opts::new(
            "message_usage_total",
            "count of messages processed by the bot",
        ),
    )
});

pub static MESSAGE_HANDLED_COUNTER: Lazy<Counter> = Lazy::new(|| {
    Counter::new(
        "message",
        Opts::new(
            "message_handled_total",
            "count of messages handled by the bot",
        ),
    )
});

pub static CMD_START_COUNTER: Lazy<Counter> = Lazy::new(|| {
    Counter::new(
        "command_start",
        Opts::new("command_start_usage_total", "count of /start invocations"),
    )
});

pub static CMD_HELP_COUNTER: Lazy<Counter> = Lazy::new(|| {
    Counter::new(
        "command_help",
        Opts::new("command_help_usage_total", "count of /help invocations"),
    )
});

pub static CMD_COUNTER: Lazy<Counter> = Lazy::new(|| {
    Counter::new(
        "commands_all",
        Opts::new("command_all_usage", "count of commands invocations"),
    )
});

pub static UNHANDLED_COUNTER: Lazy<Counter> = Lazy::new(|| {
    Counter::new(
        "unhandled",
        Opts::new("unhandled", "count of unhandled updates"),
    )
});

pub static DUEL_NUMBERS: Lazy<Counter> = Lazy::new(|| {
    Counter::new(
        "duel_numbers",
        Opts::new("duel_numbers", "Active duels on time"),
    )
});

static CPU_USAGE: Lazy<Gauge> = Lazy::new(|| {
    Gauge::new("cpu_usage", "Current CPU usage in percent").unwrap()
});

static MEM_USAGE: Lazy<Gauge> = Lazy::new(|| {
    Gauge::new("mem_usage", "Current memory usage in percent").unwrap()
});

pub fn init() -> axum::Router {
    let prometheus = REGISTRY
        .register(&INLINE_COUNTER)
        .register(&CALLBACK_COUNTER)
        .register(&MESSAGE_COUNTER)
        .register(&MESSAGE_HANDLED_COUNTER)
        .register(&UNHANDLED_COUNTER)
        .register(&CMD_START_COUNTER)
        .register(&CMD_HELP_COUNTER)
        .register(&CMD_COUNTER)
        .register(&DUEL_NUMBERS)
        .register_gauge(&CPU_USAGE)
        .register_gauge(&MEM_USAGE)
        .build();

    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    init_interval_listener();

    axum::Router::new()
        .route(
            "/metrics",
            get(|req: Request<Body>| async move {
                let headers = req.headers();
                if let Some(auth_header) = headers.get("Authorization") {
                    if let Ok(auth_str) = auth_header.to_str() {
                        if auth_str.len() > 7
                            && auth_str[7..] == BOT_CONFIG.prometheus_token
                        {
                            log::info!("Metrics: captured data");
                            let mut buffer = vec![];
                            let metrics = prometheus.gather();
                            TextEncoder::new()
                                .encode(&metrics, &mut buffer)
                                .unwrap();
                            let custom_metrics =
                                String::from_utf8(buffer).unwrap();

                            return Ok(metric_handle.render()
                                + custom_metrics.as_str());
                        }
                    }
                }
                log::warn!("Metrics: unauthorized");

                Err(axum::http::StatusCode::UNAUTHORIZED)
            }),
        )
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

                    let cpu = cpu.done().unwrap();

                    CPU_USAGE.set(f64::trunc(
                        ((cpu.system * 100.0) + (cpu.user * 100.0)).into(),
                    ));
                },
                Err(x) => log::error!("CPU load: error: {}", x),
            }

            match sys.memory() {
                Ok(mem) => {
                    let memory_used = mem.total.0 - mem.free.0;
                    let pourcentage_used =
                        (memory_used as f64 / mem.total.0 as f64) * 100.0;
                    MEM_USAGE.set(f64::trunc(pourcentage_used));
                },
                Err(x) => log::error!("Memory: error: {}", x),
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    });
}

pub struct Counter {
    inner: prometheus::Counter,
    name: String,
}

impl Counter {
    fn new(name: &str, opts: Opts) -> Counter {
        let c = prometheus::Counter::with_opts(opts)
            .unwrap_or_else(|_| panic!("unable to create {name} counter"));
        Counter { inner: c, name: name.to_string() }
    }

    #[inline]
    pub fn inc(&self) {
        self.inner.inc()
    }
}

pub struct Registry(prometheus::Registry);

impl Registry {
    fn register(&self, counter: &Counter) -> &Self {
        self.0.register(Box::new(counter.inner.clone())).unwrap_or_else(|_| {
            panic!("unable to register the {} counter", counter.name)
        });
        self
    }

    fn register_gauge(&self, gauge: &Gauge) -> &Self {
        self.0
            .register(Box::new(gauge.clone()))
            .unwrap_or_else(|_| panic!("unable to register the gauge"));

        self
    }

    fn build(&self) -> prometheus::Registry {
        self.0.clone()
    }
}
