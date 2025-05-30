//! Noop implement a [`DiceMeter`] that does nothing.

use async_trait::async_trait;

use crate::{model::dice::RolledDiceSet, services::dice::DiceMeter};

#[derive(Debug, Clone, Default)]
pub struct NoopMeter;

#[async_trait]
impl DiceMeter for NoopMeter {
    async fn register_roll(&self, _: &RolledDiceSet) {}
}
