//! This module implements the trait [`DiceMeter`] with an OpenTelemetry meter.

use async_trait::async_trait;
use opentelemetry::{
    KeyValue,
    metrics::{Histogram, Meter},
};

use crate::{model::dice::RolledDiceSet, services::dice::DiceMeter};

const ROLLED_DICE_HISTOGRAM_NAME: &str = "roll_result";
const DICE_ATTRIBUTE_KEY: &str = "";

#[derive(Debug, Clone)]
pub struct OpenTelemetryMeter {
    roll_hist: Histogram<u64>,
}

impl OpenTelemetryMeter {
    #[must_use]
    pub fn new(meter: &Meter) -> Self {
        let roll_hist = meter
            .u64_histogram(ROLLED_DICE_HISTOGRAM_NAME)
            .with_description("")
            .build();

        Self { roll_hist }
    }
}

#[async_trait]
impl DiceMeter for OpenTelemetryMeter {
    async fn register_roll(&self, rolled_dice_set: &RolledDiceSet) {
        for rolled_dice in rolled_dice_set.iter() {
            self.roll_hist.record(
                u64::from(rolled_dice.result()),
                &[KeyValue::new(
                    DICE_ATTRIBUTE_KEY,
                    rolled_dice.dice().to_string(),
                )],
            );
        }
    }
}
