//! This module provides the grounding of the Dice Service API:
//! It exposes the [`Error`]s of the service, their interface through the [`DiceService`] trait,
//! and finally the structures that are used to call this *interface*.

use async_trait::async_trait;
use std::fmt::Display;
use uuid::Uuid;

use crate::dice::Error;
use crate::dice::model::{DiceSet, RolledDiceSet};

pub mod in_memory;
pub mod noop;

#[cfg(feature = "protobuf")]
pub mod grpc;

#[cfg(feature = "opentelemetry")]
pub mod opentelemetry;

#[cfg(feature = "postgres")]
pub mod postgres;

#[async_trait]
pub trait DiceService {
    /// Roll the provided dices and save the result in the history
    ///
    /// # Errors
    ///
    /// [`Error::WayTooManyDices`] if the result can outbound [`u32`].
    async fn roll_dices(&self, req: &RollDicesRequest) -> Result<RollDicesResponse, Error>;

    /// Get the past dice roll with the given UUID
    ///
    /// # Errors
    ///
    /// [`Error::NonExistingDiceRoll`] if the provided UUID cannot be found in the repo.
    async fn get_dice_roll(&self, id: &RollId) -> Result<RollDicesResponse, Error>;
}

/// The unique identification of a dice roll.
#[derive(Debug, Clone, PartialEq)]
pub struct RollId(Uuid);

#[allow(clippy::new_without_default)]
impl RollId {
    /// Create a new randomly generated identification string. The underlying value is a UUID v7.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Parses the provided value as a UUID v7.
    ///
    /// # Errors
    ///
    /// This function will return an error if the provided value cannot be parsed as an UUID.
    pub fn parse(value: &str) -> Result<Self, Error> {
        Uuid::parse_str(value)
            .map_err(|_| Error::RollIdParseError)
            .map(RollId)
    }

    #[must_use]
    pub fn into_string(self) -> String {
        self.0.to_string()
    }
}

impl From<Uuid> for RollId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl AsRef<Uuid> for RollId {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

impl Display for RollId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Structure that holds the dice set that is meant to be rolled.
#[derive(Debug, Clone)]
pub struct RollDicesRequest {
    /// The dice set to be rolled.
    pub dice_set: DiceSet,
}

/// The result of rolling a set of dices.
#[derive(Debug, Clone)]
pub struct RollDicesResponse {
    /// The UUID that uniquely identifies the dice roll.
    pub id: RollId,

    /// The result of rolling the provided dice set.
    pub rolled_dice_set: RolledDiceSet,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DiceHistorySaver: Send + Sync + 'static {
    async fn save_roll(&self, id: &RollId, rolled_dice_set: &RolledDiceSet) -> Result<(), Error>;

    async fn get_dice_roll(&self, id: &RollId) -> Result<RolledDiceSet, Error>;
}

#[async_trait]
pub trait DiceMeter: Send + Sync + 'static {
    async fn register_roll(&self, rolled_dice_set: &RolledDiceSet);
}

#[derive(Debug)]
pub struct Service<R, M>
where
    R: DiceHistorySaver,
    M: DiceMeter,
{
    repo: R,
    meter: M,
}

impl<R, M> Service<R, M>
where
    R: DiceHistorySaver,
    M: DiceMeter,
{
    pub fn new(repo: R, meter: M) -> Self {
        Self { repo, meter }
    }
}

#[async_trait]
impl<R, M> DiceService for Service<R, M>
where
    R: DiceHistorySaver,
    M: DiceMeter,
{
    async fn roll_dices(&self, req: &RollDicesRequest) -> Result<RollDicesResponse, Error> {
        let rolled_dice_set = req.dice_set.clone().roll()?;
        let id = RollId::new();
        self.meter.register_roll(&rolled_dice_set).await;
        self.repo.save_roll(&id, &rolled_dice_set).await?;

        Ok(RollDicesResponse {
            id,
            rolled_dice_set,
        })
    }

    async fn get_dice_roll(&self, id: &RollId) -> Result<RollDicesResponse, Error> {
        self.repo
            .get_dice_roll(id)
            .await
            .map(|rolled_dice_set| RollDicesResponse {
                id: id.clone(),
                rolled_dice_set,
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::dice::model::{Dice, DiceSet};
    use crate::dice::service::{in_memory::InMemoryDiceHistorySaver, noop::NoopMeter};

    use super::*;

    #[tokio::test]
    async fn can_roll_and_fetch_dices() {
        let sut = Service::new(InMemoryDiceHistorySaver::default(), NoopMeter);

        let roll_result = sut
            .roll_dices(&RollDicesRequest {
                dice_set: DiceSet::new(vec![Dice::D20].into_iter()),
            })
            .await;

        assert!(roll_result.is_ok());

        let RollDicesResponse {
            id,
            rolled_dice_set,
        } = roll_result.unwrap();

        let query_result = sut.get_dice_roll(&id).await;
        assert!(query_result.is_ok());
        assert_eq!(query_result.unwrap().rolled_dice_set, rolled_dice_set);
    }
}
