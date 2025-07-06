use cof::services::dice;
use cof::services::dice::implem::opentelemetry::OpenTelemetryMeter;
use cof::services::dice::implem::postgres::PostgresRepo;
use opentelemetry::global;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use sqlx::postgres::PgPoolOptions;

use tonic::transport::Server;

use crate::telemetry::OpenTelemetryMonitor;

mod telemetry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let OpenTelemetryMonitor {
        logger_provider,
        meter_provider,
    } = OpenTelemetryMonitor::<SdkLoggerProvider, SdkMeterProvider>::new_with_sdk(
        opentelemetry_stdout::LogExporter::default(),
        opentelemetry_stdout::MetricExporterBuilder::default().build(),
    );

    log::info!("Starting Dice Server");

    let repo = PostgresRepo::new(
        PgPoolOptions::new()
            .connect("postgres://postgres:welcome@127.0.0.1:5432/database")
            .await?,
    )
    .await?;

    global::set_meter_provider(meter_provider.clone());
    let dice_meter = global::meter("dice_service");
    let meter = OpenTelemetryMeter::new(&dice_meter);

    let dice_svc = dice::Service::new(repo, meter);

    let addr = "0.0.0.0:50052".parse().unwrap();

    log::info!("Starting gRPC server on {addr}");

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(
            dice::implem::grpc::pb::dice_api::v1::FILE_DESCRIPTOR_SET,
        )
        .build_v1()
        .unwrap();

    Server::builder()
        .add_service(reflection_service)
        .add_service(dice_svc.into_tonic_service())
        .serve(addr)
        .await?;

    logger_provider.shutdown()?;
    meter_provider.shutdown()?;

    Ok(())
}
