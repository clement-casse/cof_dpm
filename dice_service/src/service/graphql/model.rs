use async_graphql::{InputObject, Object, ID};

use crate::service::{RollDicesRequest, RollDicesResponse};

#[derive(Debug, Clone)]
pub struct Die(crate::Die);

#[Object]
impl Die {
    async fn value(&self) -> &str {
        self.0.into()
    }
}

#[derive(Debug, Clone)]
pub struct DiceRollOutput {
    id: ID,
}

#[Object]
impl DiceRollOutput {
    async fn id(&self) -> &str {
        &self.id
    }
}

impl From<RollDicesResponse> for DiceRollOutput {
    fn from(value: RollDicesResponse) -> Self {
        Self {
            id: value.id.into(),
        }
    }
}

#[derive(Debug, Clone, InputObject)]
pub struct DiceRollInput {
    dice: String,
}

impl From<DiceRollInput> for RollDicesRequest {
    fn from(value: DiceRollInput) -> Self {
        Self {
            dice_set: crate::DiceSet(vec![]),
        }
    }
}
