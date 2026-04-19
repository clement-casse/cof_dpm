use std::sync::Arc;

use tonic::{metadata::MetadataMap, Request, Response, Status};
use tracing::{debug, error, warn};

use super::pb::dice_api::v1;
use crate::{
    repo::DiceHistorySaver, service::{Context, DiceService, RollDicesRequest, Service, CTX},
    Error,
    RollId,
};

impl<R> Service<R>
where
    R: DiceHistorySaver,
{
    /// Create a gRPC Tonic server from the actual service.
    pub fn into_tonic_service(
        self,
    ) -> v1::dice_service_server::DiceServiceServer<DiceServiceWrapper<R>> {
        let arc_service = Arc::new(self);

        v1::dice_service_server::DiceServiceServer::new(DiceServiceWrapper { svc: arc_service })
    }
}

const USER_ID_METADATA_KEY: &str = "authorization";

fn make_context_from(mm: &MetadataMap) -> Context {
    let mut cx = Context::default();

    if let Some(user_id) = mm.get(USER_ID_METADATA_KEY) {
        let user_id = user_id
            .to_str()
            .inspect_err(|err| warn!(?err, "Cannot parse user ID"))
            .unwrap_or_default();
        cx = cx.with_user(user_id);
    }

    cx
}

/// Wrapper of the [`Service`] structure that associates the gRPC methods
/// of the API to the calls of the [`Service`] methods.
/// This type allows to build a gRPC server that wraps the service.
///
/// This structure can be built from [`Service::into_tonic_service`] method.
pub struct DiceServiceWrapper<R>
where
    R: DiceHistorySaver,
{
    svc: Arc<Service<R>>,
}

#[tonic::async_trait]
impl<R> v1::dice_service_server::DiceService for DiceServiceWrapper<R>
where
    R: DiceHistorySaver,
{
    async fn roll_dices(
        &self,
        req: Request<v1::RollDicesRequest>,
    ) -> Result<Response<v1::RollDicesResponse>, Status> {
        let cx = make_context_from(req.metadata());

        let req = RollDicesRequest::try_from(req.into_inner())?;
        let resp = CTX.scope(cx, self.svc.roll_dices(&req)).await?;

        Ok(Response::new(resp.into()))
    }

    async fn get_dice_roll(
        &self,
        req: Request<v1::GetDiceRollRequest>,
    ) -> Result<Response<v1::GetDiceRollResponse>, Status> {
        let cx = make_context_from(req.metadata());

        let v1::GetDiceRollRequest { id } = req.into_inner();
        let id = RollId::parse(&id)?;
        let resp = CTX.scope(cx, self.svc.get_dice_roll(&id)).await?;

        Ok(Response::new(resp.into()))
    }
}

impl From<Error> for Status {
    fn from(value: Error) -> Self {
        match value {
            err @ (Error::DieUnknown(_) | Error::WayTooManyDices | Error::DiceSetParseError) => {
                debug!(?err);
                Self::invalid_argument("query used a malformed dice, cannot process")
            }
            err @ Error::NonExistingDiceRoll => {
                debug!(?err);
                Self::not_found("The dice roll requested cannot be found")
            }
            err => {
                error!(?err, "Error from underlying implementation");
                Self::internal("An internal error occurred")
            }
        }
    }
}
