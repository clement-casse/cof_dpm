//! Module that contains the logic of the Dice Service API.

use async_trait::async_trait;

use super::{DiceService, Error, RollDicesRequest, RollDicesResponse, RollId};
use crate::model::dice::RolledDiceSet;

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
    use crate::model::dice::{Dice, DiceSet};
    use crate::services::dice::implem::{in_memory::InMemoryDiceHistorySaver, noop::NoopMeter};

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
