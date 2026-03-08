use anyhow::Context;
use tonic::transport::Channel;

use crate::{
    Error, RollId,
    service::{DiceService, RollDicesRequest, RollDicesResponse},
};

use super::pb::dice_api::v1;

/// [`DiceService`] implementation for a remote `DiceService` served over gRPC.
/// Instead of calling the service implementation, the `DiceServiceGrpcClient` uses
/// gRPC to call a remote service.
pub struct DiceServiceGrpcClient {
    client: v1::dice_service_client::DiceServiceClient<Channel>,
}

impl DiceServiceGrpcClient {
    /// Create a new `DiceServiceGrpcClient` with the given underlying channel.
    #[must_use]
    pub fn new(channel: Channel) -> Self {
        Self {
            client: v1::dice_service_client::DiceServiceClient::new(channel),
        }
    }
}

#[tonic::async_trait]
impl DiceService for DiceServiceGrpcClient {
    async fn roll_dices(&self, req: &RollDicesRequest) -> Result<RollDicesResponse, Error> {
        let mut client = self.client.clone();
        let grpc_resp = client
            .roll_dices(v1::RollDicesRequest::from(req.clone()))
            .await
            .context("Error while getting gRPC response from RollDice")?
            .into_inner();

        Ok(RollDicesResponse::try_from(grpc_resp)
            .context("Error decoding RollDices gRPC response")?)
    }

    async fn get_dice_roll(&self, id: &RollId) -> Result<RollDicesResponse, Error> {
        let mut client = self.client.clone();
        let grpc_resp = client
            .get_dice_roll(v1::GetDiceRollRequest {
                id: id.clone().into_string(),
            })
            .await
            .context("Error while getting gRPC response from RollDice")?
            .into_inner();

        Ok(RollDicesResponse::try_from(grpc_resp)
            .context("Error decoding RollDices gRPC response")?)
    }
}
