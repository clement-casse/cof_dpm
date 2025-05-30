use regex::Regex;
use std::{collections::BTreeMap, fmt::Display, str::FromStr};

use super::{Dice, Error, RolledDice};

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
    type Err = super::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"(?<number>[0-9]+)?(?<dice>d[0-9]+)").unwrap();
        let dices: Vec<(u32, Dice)> = re
            .captures_iter(s)
            .map(|caps| {
                let number = caps
                    .name("number")
                    .map_or(Ok(1u32), |m| m.as_str().parse::<u32>())
                    .map_err(|_| super::Error::DiceSetParseError)?;

                let dice = caps.name("dice").unwrap().as_str().try_into()?;

                Ok((number, dice))
            })
            .collect::<Result<Vec<_>, Self::Err>>()?;

        if dices.is_empty() {
            return Err(super::Error::DiceSetParseError);
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
