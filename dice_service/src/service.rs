pub mod grpc;

use async_trait::async_trait;

use super::{DiceSet, Error, RollId, RolledDiceSet};
use crate::repo;

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

/// Implementation of the dice service
#[derive(Debug)]
pub struct Service<R>
where
    R: repo::DiceHistorySaver,
{
    repo: R,
}

impl<R> Service<R>
where
    R: repo::DiceHistorySaver,
{
    pub const fn new(repo: R) -> Self {
        Self {
            repo,
        }
    }
}

tokio::task_local! {
    pub static CTX: Context;
}

/// `Context` carries some pieces of information accross the multiple nested
/// function calls.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Context {
    user: Option<String>,
}

impl Context {
    #[must_use]
    pub fn with_user(self, user: &str) -> Self {
        Self {
            user: Some(user.to_string()),
            ..self
        }
    }
}

#[async_trait]
impl<R> DiceService for Service<R>
where
    R: repo::DiceHistorySaver,
{
    async fn roll_dices(&self, req: &RollDicesRequest) -> Result<RollDicesResponse, Error> {
        let rolled_dice_set = req.dice_set.clone().roll()?;
        let id = RollId::new();
        self.repo.save_roll(&id, &rolled_dice_set).await?;

        Ok(RollDicesResponse {
            id,
            rolled_dice_set,
        })
    }

    async fn get_dice_roll(&self, id: &RollId) -> Result<RollDicesResponse, Error> {
        self.repo.get_dice_roll(id).await.map(|rolled_dice_set| RollDicesResponse {
            id: id.clone(),
            rolled_dice_set,
        })
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
