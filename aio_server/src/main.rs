use std::sync::Arc;

use anyhow::Result;
use dice_service::{
    repo::postgres::PostgresRepo,
    service::{Service, grpc::pb::dice_api},
};
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .compact()
        .init();

    info!("Starting All-In-One Server");

    let pg_uri = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:welcome@localhost/database".to_string());

    let pg_pool = Arc::new(sqlx::PgPool::connect(&pg_uri).await?);
    info!("Connected to Postgres Database");

    let dice_service = Service::new(PostgresRepo::new_from_arc(pg_pool).await?);

    let reflection_svc = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(dice_api::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    let dice_grpc_service = dice_service.into_tonic_service();

    Server::builder()
        .add_service(reflection_svc)
        .add_service(dice_grpc_service)
        .serve("0.0.0.0:50051".parse()?)
        .await?;

    Ok(())
}
