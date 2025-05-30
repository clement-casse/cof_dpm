//! This module represents the traditional dices encountered in classical Table Top Role
//! Playing Games: there are multiples types of [`Dice`]s: *d3*, *d4*, *d6*, *d8*, *d10*,
//! *d12*, *d20* and *d100*.
//!
//! These dices can grouped together in a [`DiceSet`] and, above everything else, they can
//! be rolled. Both [`Dice`] and [`DiceSet`] implement the `roll()` method that generates a
//! random value between 1 and the number of faces of the dice.
//!
//! Once dices are rolled they are instances of the [`RolledDice`] structure that provides
//! acces to the original dice and the outcome of the stochastic experience of rolling a dice
//! through the `result()` method.

mod dice_type;
pub use dice_type::*;

mod dice_set;
pub use dice_set::*;

#[cfg(feature = "protobuf")]
mod protobuf;

/// It is important that the module structure within the `pb` module matches the
/// protobuf package structure. Indeed, to take benefits of the business logic
/// implemented in this crate, the [`crate::dice::protobuf`] defines translations
/// from prost! structures into the ones defined in this crate and vice versa.
#[cfg(feature = "protobuf")]
pub mod pb {
    pub mod common {
        pub mod dice {
            pub mod v1 {
                tonic::include_proto!("cof.common.dice.v1");
            }
        }
    }
}

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Dice {0} does not exist")]
    DiceUnknown(String),

    #[error("How would a normal human put this amount of dices in a real table top game ?!?")]
    WayTooManyDices,

    #[error("Cannot parse the diceset")]
    DiceSetParseError,

    #[cfg(feature = "protobuf")]
    #[error("Received an unspecifed Protobuf value")]
    UnspecifiedProtoEnum,

    #[cfg(feature = "protobuf")]
    #[error(transparent)]
    ProstUnknownEnumValue(#[from] prost::UnknownEnumValue),
}
