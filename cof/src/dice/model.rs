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

#[cfg(feature = "protobuf")]
pub mod protobuf;

use super::Error;
use rand::prelude::*;
use regex::Regex;
use std::{collections::BTreeMap, fmt::Display, str::FromStr};

/// Dice represents the different kinds of Table Top Role Playing Games.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Dice {
    D3 = 3,
    D4 = 4,
    D6 = 6,
    D8 = 8,
    D10 = 10,
    D12 = 12,
    D20 = 20,
    D100 = 100,
}

impl Dice {
    /// returns the total number of side the dice has.
    #[must_use]
    pub fn side_count(&self) -> u32 {
        *self as u32
    }

    /// rolls the dice and returns a [`RolledDice`] containing the actual dice and
    /// the result of the roll being an integer being between 1 and the number of
    /// side the dice has.
    #[must_use]
    pub fn roll(self) -> RolledDice {
        RolledDice {
            dice: self,
            result: rand::rng().random_range(1..=self.side_count()),
        }
    }
}

impl TryFrom<&str> for Dice {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "d3" => Ok(Self::D3),
            "d4" => Ok(Self::D4),
            "d6" => Ok(Self::D6),
            "d8" => Ok(Self::D8),
            "d10" => Ok(Self::D10),
            "d12" => Ok(Self::D12),
            "d20" => Ok(Self::D20),
            "d100" => Ok(Self::D100),
            _ => Err(Self::Error::DiceUnknown(value.to_string())),
        }
    }
}

impl From<Dice> for &str {
    fn from(value: Dice) -> Self {
        match value {
            Dice::D3 => "d3",
            Dice::D4 => "d4",
            Dice::D6 => "d6",
            Dice::D8 => "d8",
            Dice::D10 => "d10",
            Dice::D12 => "d12",
            Dice::D20 => "d20",
            Dice::D100 => "d100",
        }
    }
}

impl Display for Dice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Dice::D3 => write!(f, "d3"),
            Dice::D4 => write!(f, "d4"),
            Dice::D6 => write!(f, "d6"),
            Dice::D8 => write!(f, "d8"),
            Dice::D10 => write!(f, "d10"),
            Dice::D12 => write!(f, "d12"),
            Dice::D20 => write!(f, "d20"),
            Dice::D100 => write!(f, "d100"),
        }
    }
}

/// A `RolledDice` represents the outcome of rolling a dice.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RolledDice {
    pub(super) dice: Dice,
    pub(super) result: u32,
}

impl RolledDice {
    #[must_use]
    pub fn new(dice: Dice, result: u32) -> Self {
        Self { dice, result }
    }

    /// `dice` returns the [`Dice`] that has been rolled.
    #[must_use]
    pub fn dice(&self) -> Dice {
        self.dice
    }

    /// `result` returns the result of having rolled the given dice.
    #[must_use]
    pub fn result(&self) -> u32 {
        self.result
    }
}

/// A `DiceSet` represents multiple dices to roll.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiceSet(pub(super) Vec<Dice>);

impl DiceSet {
    /// Creates a new `DiceSet` from multiple dices.
    pub fn new(dices: impl Iterator<Item = Dice>) -> Self {
        Self(dices.collect::<Vec<Dice>>())
    }

    /// Returns the lowest possible outcome for this [`DiceSet`] (i.e. all dice roll 1)
    /// (i.e. the number of the [`Dice`] in the [`DiceSet`]).
    ///
    /// # Errors
    /// [`Error::WayTooManyDices`] is returned when the result cannot be casted in [`u32`].
    pub fn lower_bound(&self) -> Result<u32, Error> {
        u32::try_from(self.0.len()).map_err(|_| Error::WayTooManyDices)
    }

    /// Returns the highest possible outcome for this [`DiceSet`] (i.e. all dice roll their
    /// maximum value) (i.e. the number of the [`Dice`] in the [`DiceSet`]).
    ///
    /// # Errors
    /// [`Error::WayTooManyDices`] is returned when the result cannot be casted in [`u32`].
    pub fn upper_bound(&self) -> Result<u32, Error> {
        self.0
            .iter()
            .try_fold(0u32, |acc, dice| acc.checked_add(*dice as u32))
            .ok_or(Error::WayTooManyDices)
    }

    /// Rolls all the dices in the `DiceSet` and returns a [`RolledDiceSet`].
    ///
    /// # Errors
    /// [`Error::WayTooManyDices`] is returned when the result cannot be casted in [`u32`].
    pub fn roll(self) -> Result<RolledDiceSet, Error> {
        if self.upper_bound().is_err() {
            return Err(Error::WayTooManyDices);
        }
        Ok(RolledDiceSet(self.0.into_iter().map(Dice::roll).collect()))
    }

    /// `iter()` returns an iterator of all the `Dice`s in the `DiceSet`.
    pub fn iter(&self) -> impl Iterator<Item = &Dice> {
        self.0.iter()
    }
}

impl FromStr for DiceSet {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"(?<number>[0-9]+)?(?<dice>d[0-9]+)").unwrap();
        let dices: Vec<(u32, Dice)> = re
            .captures_iter(s)
            .map(|caps| {
                let number = caps
                    .name("number")
                    .map_or(Ok(1u32), |m| m.as_str().parse::<u32>())
                    .map_err(|_| Error::DiceSetParseError)?;

                let dice = caps.name("dice").unwrap().as_str().try_into()?;

                Ok((number, dice))
            })
            .collect::<Result<Vec<_>, Self::Err>>()?;

        if dices.is_empty() {
            return Err(Error::DiceSetParseError);
        }

        let dice_iter = dices
            .into_iter()
            .flat_map(|(number, dice)| (0..number).map(move |_| dice));

        Ok(DiceSet::new(dice_iter))
    }
}

impl Display for DiceSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dice_counts = BTreeMap::new();
        for d in &self.0 {
            dice_counts
                .entry(*d)
                .and_modify(|e| *e += 1)
                .or_insert(1u32);
        }
        let str_content = dice_counts
            .iter()
            .rev()
            .map(|(dice, count)| {
                if *count == 1 {
                    format!("{dice}")
                } else {
                    format!("{count}{dice}")
                }
            })
            .collect::<Vec<String>>()
            .join(" + ");
        write!(f, "{str_content}")
    }
}

/// A `RolledDiceSet` represents the outcome of rolling all dices in a [`DiceSet`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RolledDiceSet(pub(super) Vec<RolledDice>);

impl RolledDiceSet {
    /// Create a new `RolledDiceSet` out of the given `RolledDice`s.
    pub fn new(rolled_dices: impl Iterator<Item = RolledDice>) -> Self {
        Self(rolled_dices.collect::<Vec<RolledDice>>())
    }

    /// `total` returns the sum of all the results of the rolls of each dices in the
    /// `RolledDiceSet`.
    #[must_use]
    pub fn total(&self) -> u32 {
        self.0.iter().fold(0u32, |acc, e| acc + e.result)
    }

    /// `iter` returns an iterator of all the `RolledDice` in the `RolledDiceSet`.
    pub fn iter(&self) -> impl Iterator<Item = &RolledDice> {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_count_sides() {
        let test_cases = &[
            (Dice::D3, 3u32),
            (Dice::D4, 4),
            (Dice::D6, 6),
            (Dice::D8, 8),
            (Dice::D10, 10),
            (Dice::D12, 12),
            (Dice::D20, 20),
            (Dice::D100, 100),
        ];

        for tc in test_cases {
            let side_count = tc.0.side_count();
            assert_eq!(side_count, tc.1);
        }
    }

    #[test]
    fn can_roll_dice() {
        const TIMES_ROLLED: u32 = 1000;

        let test_cases = &[
            (Dice::D3, 3u32),
            (Dice::D4, 4),
            (Dice::D6, 6),
            (Dice::D8, 8),
            (Dice::D10, 10),
            (Dice::D12, 12),
            (Dice::D20, 20),
            (Dice::D100, 100),
        ];

        for tc in test_cases {
            for _ in 0..TIMES_ROLLED {
                let rolled_dice = tc.0.roll();
                assert_eq!(rolled_dice.dice, tc.0);
                assert!(rolled_dice.result >= 1u32 && rolled_dice.result <= tc.1);
            }
        }
    }

    #[test]
    fn can_understand_dice_notation() {
        let test_cases = &[
            ("d3", Dice::D3),
            ("d4", Dice::D4),
            ("d6", Dice::D6),
            ("d8", Dice::D8),
            ("d10", Dice::D10),
            ("d12", Dice::D12),
            ("d20", Dice::D20),
            ("d100", Dice::D100),
        ];
        for tc in test_cases {
            let result = Dice::try_from(tc.0);
            assert!(result.is_ok());
            let this_dice = result.unwrap();
            assert_eq!(this_dice, tc.1);

            let notation: &str = tc.1.into();
            assert_eq!(notation, tc.0);
        }

        let invalid_cases = &["d", "d2", "d13", "dd", "1d20"];
        for tc in invalid_cases {
            let result = Dice::try_from(*tc);
            assert!(result.is_err());
            let this_error = result.unwrap_err();
            assert!(matches!(this_error, super::Error::DiceUnknown(_)));
        }
    }

    #[test]
    fn can_print_dices() {
        let test_cases = &[
            (Dice::D3, "1d3 + 3"),
            (Dice::D4, "1d4 + 3"),
            (Dice::D6, "1d6 + 3"),
            (Dice::D8, "1d8 + 3"),
            (Dice::D10, "1d10 + 3"),
            (Dice::D12, "1d12 + 3"),
            (Dice::D20, "1d20 + 3"),
            (Dice::D100, "1d100 + 3"),
        ];
        for tc in test_cases {
            let formatted_string = format!("1{} + 3", tc.0);
            assert_eq!(formatted_string, tc.1.to_string());
        }
    }
    #[test]
    fn can_create_dice_set() {
        let my_dice_set = DiceSet::new(
            vec!["d3", "d100", "d20", "d10", "d100", "d100"]
                .into_iter()
                .map(|e| Dice::try_from(e).unwrap()),
        );

        assert_eq!(my_dice_set.0.len(), 6);
        assert_eq!(my_dice_set.0.first(), Some(&Dice::D3));
        assert_eq!(my_dice_set.0.get(1), Some(&Dice::D100));
        assert_eq!(my_dice_set.0.get(2), Some(&Dice::D20));
        assert_eq!(my_dice_set.0.get(3), Some(&Dice::D10));
        assert_eq!(my_dice_set.0.get(4), Some(&Dice::D100));
        assert_eq!(my_dice_set.0.get(5), Some(&Dice::D100));
    }

    #[test]
    fn can_compute_diceset_lower_bound() {
        let my_dice_set = DiceSet::new(
            vec!["d3", "d100", "d20", "d10", "d100", "d100"]
                .into_iter()
                .map(|e| Dice::try_from(e).unwrap()),
        );
        assert_eq!(my_dice_set.lower_bound().unwrap(), 6);

        let my_empty_dice_set =
            DiceSet::new(vec![].into_iter().map(|e: &str| Dice::try_from(e).unwrap()));
        assert_eq!(my_empty_dice_set.lower_bound().unwrap(), 0);
    }

    #[test]
    fn can_compute_diceset_upper_bound() {
        let my_dice_set = DiceSet::new(
            vec!["d3", "d100", "d20", "d10", "d100", "d100"]
                .into_iter()
                .map(|e| Dice::try_from(e).unwrap()),
        );
        assert_eq!(my_dice_set.upper_bound().unwrap(), 333);

        let my_empty_dice_set =
            DiceSet::new(vec![].into_iter().map(|e: &str| Dice::try_from(e).unwrap()));
        assert_eq!(my_empty_dice_set.upper_bound().unwrap(), 0);
    }

    #[test]
    fn can_roll_diceset() {
        let my_dice_set = DiceSet::new(
            vec!["d3", "d100", "d20", "d10", "d100", "d100"]
                .into_iter()
                .map(|e| Dice::try_from(e).unwrap()),
        );
        let result = my_dice_set.clone().roll().unwrap();
        assert_eq!(result.0.len(), 6);
        let total = result.total();
        assert!(
            total <= my_dice_set.upper_bound().unwrap()
                && total >= my_dice_set.lower_bound().unwrap()
        );

        let my_empty_dice_set =
            DiceSet::new(vec![].into_iter().map(|e: &str| Dice::try_from(e).unwrap()));
        assert_eq!(my_empty_dice_set.roll().unwrap().0.len(), 0);
    }

    #[test]
    fn can_decode_diceset_from_str() {
        let valid_test_cases = &[
            ("d100", DiceSet::new(vec![Dice::D100].into_iter())),
            ("2d10", DiceSet::new(vec![Dice::D10, Dice::D10].into_iter())),
            (
                "3d4",
                DiceSet::new(vec![Dice::D4, Dice::D4, Dice::D4].into_iter()),
            ),
            (
                "d100 + 2d20",
                DiceSet::new(vec![Dice::D100, Dice::D20, Dice::D20].into_iter()),
            ),
        ];
        for tc in valid_test_cases {
            let ds = DiceSet::from_str(tc.0);
            assert!(ds.is_ok());
            let ds = ds.unwrap();
            assert_eq!(ds, tc.1);
        }

        let error_cases = &["2d7", "D100"];
        for tc in error_cases {
            let ds = DiceSet::from_str(tc);
            assert!(ds.is_err());
        }
    }

    #[test]
    fn can_display_diceset() {
        let valid_test_cases = &[
            (DiceSet::new(vec![Dice::D100].into_iter()), "d100"),
            (DiceSet::new(vec![Dice::D10, Dice::D10].into_iter()), "2d10"),
            (
                DiceSet::new(vec![Dice::D4, Dice::D4, Dice::D4].into_iter()),
                "3d4",
            ),
            (
                DiceSet::new(vec![Dice::D100, Dice::D20, Dice::D20].into_iter()),
                "d100 + 2d20",
            ),
            (
                DiceSet::new(vec![Dice::D20, Dice::D20, Dice::D100].into_iter()),
                "d100 + 2d20",
            ),
            (
                DiceSet::new(vec![Dice::D20, Dice::D100, Dice::D20].into_iter()),
                "d100 + 2d20",
            ),
            (
                DiceSet::new(
                    vec![
                        Dice::D20,
                        Dice::D100,
                        Dice::D20,
                        Dice::D8,
                        Dice::D100,
                        Dice::D20,
                    ]
                    .into_iter(),
                ),
                "2d100 + 3d20 + d8",
            ),
        ];

        for tc in valid_test_cases {
            assert_eq!(tc.0.to_string(), tc.1.to_string());
        }
    }
}
