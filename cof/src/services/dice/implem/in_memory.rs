//! Module providing In-Memory Adapter for the [`crate::services::dice::service::DiceHistorySaver`].
//! This adapter main use is for tests and protoyping and does not perform long-lasting storage.

use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::model::dice::RolledDiceSet;
use crate::services::dice::service::DiceHistorySaver;
use crate::services::dice::{Error, RollId};

#[derive(Debug, Default)]
pub struct InMemoryDiceHistorySaver {
    repo: RwLock<HashMap<Uuid, RolledDiceSet>>,
}

#[async_trait]
impl DiceHistorySaver for InMemoryDiceHistorySaver {
    async fn save_roll(&self, id: &RollId, rolled_dice_set: &RolledDiceSet) -> Result<(), Error> {
        let mut hm = self.repo.write().await;
        hm.entry(id.0).insert_entry(rolled_dice_set.clone());
        Ok(())
    }

    async fn get_dice_roll(&self, id: &RollId) -> Result<RolledDiceSet, Error> {
        let hm = self.repo.read().await;
        let rolled_dice_set = hm.get(&id.0).ok_or(Error::NonExistingDiceRoll)?;
        Ok(rolled_dice_set.clone())
    }
}
