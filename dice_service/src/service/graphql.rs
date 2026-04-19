mod model;

use std::sync::Arc;

use async_graphql::{Context, EmptySubscription, Error, ErrorExtensions, Object, Schema, ID};

use crate::{
    service::{DiceService, RollDicesRequest}, DiceSet, Error as ServiceError,
    RollId,
};

impl ErrorExtensions for crate::Error {
    fn extend(&self) -> Error {
        Error::new(self.to_string()).extend_with(|err, e| match self {
            ServiceError::DieUnknown(_)
            | ServiceError::WayTooManyDices
            | ServiceError::DiceSetParseError
            | ServiceError::RollIdParseError => e.set("status", "BAD_REQUEST"),
            ServiceError::NonExistingDiceRoll => e.set("status", "NOT_FOUND"),
            _ => e.set("status", "INTERNAL"),
        })
    }
}

pub type DiceServiceSchema = Schema<DiceServiceQuery, DiceServiceMutation, EmptySubscription>;

#[derive(Debug, Default)]
pub struct DiceServiceQuery;

#[Object]
impl DiceServiceQuery {
    async fn get_dice_roll(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> Result<model::DiceRollOutput, Error> {
        let svc = ctx.data::<Arc<dyn DiceService>>()?;

        let roll_id = RollId::parse(id.as_str())?;
        let resp = svc.get_dice_roll(&roll_id).await?;

        Ok(resp.into())
    }
}

#[derive(Debug, Default)]
pub struct DiceServiceMutation;

#[Object]
impl DiceServiceMutation {
    async fn roll_dice(
        &self,
        ctx: &Context<'_>,
        input: model::DiceRollInput,
    ) -> Result<model::DiceRollOutput, Error> {
        let svc = ctx.data::<Arc<dyn DiceService>>()?;

        let req = RollDicesRequest {
            dice_set: DiceSet(vec![]),
        };
        let resp = svc.roll_dices(&req).await?;

        Ok(resp.into())
    }
}
