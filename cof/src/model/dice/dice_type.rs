use rand::prelude::*;
use std::fmt::Display;

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
    type Error = super::Error;

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
            assert!(matches!(
                this_error,
                crate::model::dice::Error::DiceUnknown(_)
            ));
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
}
