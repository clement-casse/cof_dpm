//! This module provides the grounding of the Dice Service API:
//! It exposes the [`Error`]s of the service, their interface through the [`DiceService`] trait,
//! and finally the structures that are used to call this *interface*.

use std::fmt::Display;

use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

use crate::model::dice::{DiceSet, Error as DiceError, RolledDiceSet};

mod service;
pub use service::*;

pub mod implem;

#[derive(Debug, Error)]
pub enum Error {
    #[error("The given dice roll cannot be found")]
    NonExistingDiceRoll,

    #[error("The provided Roll ID cannot be parsed")]
    RollIdParseError,

    #[error(transparent)]
    FromModel(#[from] DiceError),

    #[error(transparent)]
    Underlying(#[from] anyhow::Error),
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct RollId(Uuid);

#[allow(clippy::new_without_default)]
impl RollId {
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
