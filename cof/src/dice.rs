pub mod model;

pub mod service;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Dice {0} does not exist")]
    DiceUnknown(String),

    #[error(
        "How would a human would decently put this amount of dices in a real table top game ?!?"
    )]
    WayTooManyDices,

    #[error("Cannot parse the diceset")]
    DiceSetParseError,

    #[error("The given dice roll cannot be found")]
    NonExistingDiceRoll,

    #[error("The provided Roll ID cannot be parsed")]
    RollIdParseError,

    #[error(transparent)]
    Underlying(#[from] anyhow::Error),
}
