use opentelemetry::{logs::LoggerProvider, metrics::MeterProvider};
use opentelemetry_appender_tracing::layer;
use opentelemetry_sdk::{
    Resource,
    logs::{LogExporter, SdkLoggerProvider},
    metrics::{SdkMeterProvider, exporter::PushMetricExporter},
};
use tracing_subscriber::{EnvFilter, prelude::*};

const SERVICE_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Debug)]
pub struct OpenTelemetryMonitor<L: LoggerProvider, M: MeterProvider> {
    pub logger_provider: L,
    pub meter_provider: M,
}

impl<L, M> OpenTelemetryMonitor<L, M>
where
    L: LoggerProvider,
    M: MeterProvider,
{
    pub fn new_with_sdk(
        log_exporter: impl LogExporter + 'static,
        metric_exporter: impl PushMetricExporter,
    ) -> OpenTelemetryMonitor<SdkLoggerProvider, SdkMeterProvider> {
        let resource = Resource::builder().with_service_name(SERVICE_NAME).build();

        let logger_provider = SdkLoggerProvider::builder()
            .with_resource(resource.clone())
            .with_simple_exporter(log_exporter)
            .build();

        // Remove all dependency logs from data send via Opentelemetry
        let filter_otel_tracing_bridge = EnvFilter::from_default_env()
            .add_directive("opentelemetry=off".parse().unwrap())
            .add_directive("hyper=off".parse().unwrap())
            .add_directive("tonic=off".parse().unwrap())
            .add_directive("h2=off".parse().unwrap())
            .add_directive("reqwest=off".parse().unwrap())
            .add_directive("sqlx=off".parse().unwrap());

        // Force INFO level on dependencies
        let filter_fmt = EnvFilter::from_default_env()
            .add_directive("opentelemetry=info".parse().unwrap())
            .add_directive("hyper=info".parse().unwrap())
            .add_directive("tonic=info".parse().unwrap())
            .add_directive("h2=info".parse().unwrap())
            .add_directive("reqwest=info".parse().unwrap())
            .add_directive("sqlx=info".parse().unwrap());

        tracing_subscriber::registry()
            .with(
                layer::OpenTelemetryTracingBridge::new(&logger_provider)
                    .with_filter(filter_otel_tracing_bridge),
            )
            .with(tracing_subscriber::fmt::layer().with_filter(filter_fmt))
            .init();

        let meter_provider = SdkMeterProvider::builder()
            .with_periodic_exporter(metric_exporter)
            .with_resource(resource.clone())
            .build();

        OpenTelemetryMonitor::<SdkLoggerProvider, SdkMeterProvider> {
            logger_provider,
            meter_provider,
        }
    }
}
