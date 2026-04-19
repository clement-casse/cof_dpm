//! Module providing In-Memory Adapter for the [`DiceHistorySaver`].
//! This adapter main use is for tests and prototyping and does not perform long-lasting storage.

use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::DiceHistorySaver;
use crate::{Error, RollId, RolledDiceSet};

#[derive(Debug, Default)]
pub struct InMemoryDiceHistorySaver {
    repo: RwLock<HashMap<Uuid, RolledDiceSet>>,
}

#[async_trait]
impl DiceHistorySaver for InMemoryDiceHistorySaver {
    async fn save_roll(&self, id: &RollId, rolled_dice_set: &RolledDiceSet) -> Result<(), Error> {
        {
            let mut hm = self.repo.write().await;
            hm.entry(id.0).insert_entry(rolled_dice_set.clone());
        }

        Ok(())
    }

    async fn get_dice_roll(&self, id: &RollId) -> Result<RolledDiceSet, Error> {
        let rolled_dice_set = {
            let hm = self.repo.read().await;
            hm.get(&id.0).cloned().ok_or(Error::NonExistingDiceRoll)?
        };

        Ok(rolled_dice_set)
    }
}
