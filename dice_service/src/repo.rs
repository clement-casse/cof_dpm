pub mod in_memory;

#[cfg(feature = "postgres")]
pub mod postgres;

use async_trait::async_trait;

use super::{Error, RollId, RolledDiceSet};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DiceHistorySaver: Send + Sync + 'static {
    async fn save_roll(&self, id: &RollId, rolled_dice_set: &RolledDiceSet) -> Result<(), Error>;

    async fn get_dice_roll(&self, id: &RollId) -> Result<RolledDiceSet, Error>;
}
