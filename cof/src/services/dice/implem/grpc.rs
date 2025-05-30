//! This module provides the protobuf encoding and decoding methods as well as trivial Tonic
//! client and servers wrappers to call the remote service exactly as the local one.

use anyhow::Context;
use log::error;
use tonic::{Request, Response, Status, transport::Channel};

use crate::model::dice::RolledDiceSet;
use crate::services::dice::{
    DiceHistorySaver, DiceMeter, DiceService, Error, RollDicesRequest, RollDicesResponse, RollId,
    Service,
};

/// Module that contains the Prost! code generation for the dice API.
pub mod pb {
    pub use crate::model::dice::pb::common;
    pub mod dice_api {
        pub mod v1 {
            tonic::include_proto!("cof.dice_api.v1");

            pub const FILE_DESCRIPTOR_SET: &[u8] =
                tonic::include_file_descriptor_set!("diceapiv1_descriptor");
        }
    }
}

use pb::dice_api::v1;

/// Wrapper of the [`super::Service`] structure that associates the gRPC methods
/// of the API to the calls of the [`super::Service`] methods.
/// This type allows to build a gRPC server that wraps the service.
///
/// This structure can be build from [`Service::into_tonic_service`] method.
pub struct DiceServiceWrapper<R, M>
where
    R: DiceHistorySaver,
    M: DiceMeter,
{
    svc: Service<R, M>,
}

#[tonic::async_trait]
impl<R, M> v1::dice_service_server::DiceService for DiceServiceWrapper<R, M>
where
    R: DiceHistorySaver,
    M: DiceMeter,
{
    async fn roll_dices(
        &self,
        request: Request<v1::RollDicesRequest>,
    ) -> Result<Response<v1::RollDicesResponse>, Status> {
        let req = RollDicesRequest::try_from(request.into_inner()).map_err(Error::Underlying)?;
        let resp = self.svc.roll_dices(&req).await?;

        Ok(Response::new(resp.into()))
    }

    async fn get_dice_roll(
        &self,
        request: Request<v1::GetDiceRollRequest>,
    ) -> Result<Response<v1::GetDiceRollResponse>, Status> {
        let v1::GetDiceRollRequest { id } = request.into_inner();
        let id = RollId::parse(&id)?;
        let resp = self.svc.get_dice_roll(&id).await?;

        Ok(Response::new(resp.into()))
    }
}

impl From<Error> for Status {
    fn from(value: Error) -> Self {
        match value {
            Error::NonExistingDiceRoll => {
                Status::not_found("The dice roll requested cannot be found")
            }
            Error::RollIdParseError => Status::internal("Something went wrong"),
            Error::FromModel(error) => {
                error!("Error from model: {error:?}");
                Status::failed_precondition("")
            }
            Error::Underlying(error) => {
                error!("Error from underlying implementation: {error:?}");
                Status::internal("An internal error occured")
            }
        }
    }
}

impl<R, M> Service<R, M>
where
    R: DiceHistorySaver,
    M: DiceMeter,
{
    /// Create a gRPC Tonic server from the actual service.
    pub fn into_tonic_service(
        self,
    ) -> v1::dice_service_server::DiceServiceServer<DiceServiceWrapper<R, M>> {
        v1::dice_service_server::DiceServiceServer::new(DiceServiceWrapper { svc: self })
    }
}

/// [`super::DiceService`] implementation for a remote `DiceService` served over gRPC.
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

impl From<RollDicesRequest> for v1::RollDicesRequest {
    fn from(value: RollDicesRequest) -> Self {
        let dices = value
            .dice_set
            .iter()
            .map(|d| pb::common::dice::v1::DiceType::from(*d) as i32)
            .collect();

        Self { dices }
    }
}

impl TryFrom<v1::RollDicesRequest> for RollDicesRequest {
    type Error = anyhow::Error;

    fn try_from(value: v1::RollDicesRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            dice_set: value
                .dices()
                .collect::<Vec<_>>()
                .try_into()
                .context("Cannot parse DiceSet")?,
        })
    }
}

impl From<RollDicesResponse> for v1::RollDicesResponse {
    fn from(value: RollDicesResponse) -> Self {
        Self {
            id: value.id.to_string(),
            rolled_dices: value.rolled_dice_set.into(),
        }
    }
}

impl TryFrom<v1::RollDicesResponse> for RollDicesResponse {
    type Error = anyhow::Error;

    fn try_from(value: v1::RollDicesResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            id: RollId::parse(&value.id).context("Cannot parse UUID")?,
            rolled_dice_set: RolledDiceSet::try_from(value.rolled_dices)
                .context("Cannot parse the resulting dice set")?,
        })
    }
}

impl From<RollDicesResponse> for v1::GetDiceRollResponse {
    fn from(value: RollDicesResponse) -> Self {
        Self {
            id: value.id.to_string(),
            rolled_dices: value.rolled_dice_set.into(),
        }
    }
}

impl TryFrom<v1::GetDiceRollResponse> for RollDicesResponse {
    type Error = anyhow::Error;

    fn try_from(value: v1::GetDiceRollResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            id: RollId::parse(&value.id).context("Cannot parse UUID")?,
            rolled_dice_set: RolledDiceSet::try_from(value.rolled_dices)
                .context("Cannot parse the resulting dice set")?,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::dice::{Dice, DiceSet};
    use crate::services::dice::{
        RollDicesRequest,
        implem::{in_memory::InMemoryDiceHistorySaver, noop::NoopMeter},
    };

    #[test]
    fn can_encode_and_decode_dice_roll_requests() {
        let dice_set = DiceSet::new(vec![Dice::D100].into_iter());
        let req = RollDicesRequest {
            dice_set: dice_set.clone(),
        };

        let proto_req = v1::RollDicesRequest::from(req);

        assert_eq!(proto_req.dices.len(), 1);
        assert!(
            proto_req
                .dices()
                .all(|d| d == pb::common::dice::v1::DiceType::DiceType100)
        );

        let initial_req = RollDicesRequest::try_from(proto_req);
        assert!(initial_req.is_ok());

        assert_eq!(initial_req.unwrap().dice_set, dice_set);
    }

    #[tokio::test]
    async fn can_encode_and_decode_dice_roll_response() {
        let svc = Service::new(InMemoryDiceHistorySaver::default(), NoopMeter);

        let roll_dice_resp = svc
            .roll_dices(&RollDicesRequest {
                dice_set: DiceSet::new(vec![Dice::D100].into_iter()),
            })
            .await
            .unwrap();

        let proto_roll_resp = v1::RollDicesResponse::from(roll_dice_resp.clone());
        assert_eq!(proto_roll_resp.rolled_dices.len(), 1);
        assert_eq!(
            proto_roll_resp.rolled_dices[0].dice(),
            pb::common::dice::v1::DiceType::DiceType100
        );

        let get_rolled_dice_resp = svc.get_dice_roll(&roll_dice_resp.id).await.unwrap();
        assert_eq!(get_rolled_dice_resp.id, roll_dice_resp.id);
        assert_eq!(
            get_rolled_dice_resp.rolled_dice_set,
            roll_dice_resp.rolled_dice_set
        );

        let proto_roll_resp = v1::GetDiceRollResponse::from(get_rolled_dice_resp.clone());
        assert_eq!(proto_roll_resp.rolled_dices.len(), 1);
        assert_eq!(
            proto_roll_resp.rolled_dices[0].dice(),
            pb::common::dice::v1::DiceType::DiceType100
        );
    }
}
