pub mod client;
pub mod server;

pub mod pb {
    pub mod dice_api {
        pub mod v1 {
            tonic::include_proto!("cof.dice_api.v1");

            pub const FILE_DESCRIPTOR_SET: &[u8] =
                tonic::include_file_descriptor_set!("diceapiv1_descriptor");
        }
    }
}

use anyhow::{Context as _, anyhow};
use pb::dice_api::v1;

use crate::{
    Dice, DiceSet, Error, RollId, RolledDice, RolledDiceSet,
    service::{RollDicesRequest, RollDicesResponse},
};

impl From<Dice> for v1::DiceType {
    fn from(value: Dice) -> Self {
        match value {
            Dice::D3 => Self::DiceType3,
            Dice::D4 => Self::DiceType4,
            Dice::D6 => Self::DiceType6,
            Dice::D8 => Self::DiceType8,
            Dice::D10 => Self::DiceType10,
            Dice::D12 => Self::DiceType12,
            Dice::D20 => Self::DiceType20,
            Dice::D100 => Self::DiceType100,
        }
    }
}

impl TryFrom<v1::DiceType> for Dice {
    type Error = Error;

    fn try_from(value: v1::DiceType) -> Result<Self, Self::Error> {
        match value {
            v1::DiceType::DiceType3 => Ok(Self::D3),
            v1::DiceType::DiceType4 => Ok(Self::D4),
            v1::DiceType::DiceType6 => Ok(Self::D6),
            v1::DiceType::DiceType8 => Ok(Self::D8),
            v1::DiceType::DiceType10 => Ok(Self::D10),
            v1::DiceType::DiceType12 => Ok(Self::D12),
            v1::DiceType::DiceType20 => Ok(Self::D20),
            v1::DiceType::DiceType100 => Ok(Self::D100),
            v1::DiceType::Unspecified => Err(Self::Error::Underlying(anyhow!(
                "the value of the protobuf DiceType was UNSPECIFIED"
            ))),
        }
    }
}

impl From<DiceSet> for Vec<v1::DiceType> {
    fn from(value: DiceSet) -> Self {
        value.0.into_iter().map(v1::DiceType::from).collect()
    }
}

impl TryFrom<Vec<v1::DiceType>> for DiceSet {
    type Error = Error;

    fn try_from(value: Vec<v1::DiceType>) -> Result<Self, Self::Error> {
        let encoded_dices = value.into_iter().map(Dice::try_from).collect::<Result<Vec<_>, _>>()?;

        Ok(Self(encoded_dices))
    }
}

impl From<RolledDice> for v1::RolledDice {
    fn from(value: RolledDice) -> Self {
        Self {
            dice: v1::DiceType::from(value.dice) as i32,
            result: value.result,
        }
    }
}

impl TryFrom<v1::RolledDice> for RolledDice {
    type Error = Error;

    fn try_from(value: v1::RolledDice) -> Result<Self, Self::Error> {
        let dice = Dice::try_from(value.dice())?;
        Ok(Self {
            dice,
            result: value.result,
        })
    }
}

impl From<RolledDiceSet> for Vec<v1::RolledDice> {
    fn from(value: RolledDiceSet) -> Self {
        value.0.into_iter().map(v1::RolledDice::from).collect()
    }
}

impl TryFrom<Vec<v1::RolledDice>> for RolledDiceSet {
    type Error = Error;

    fn try_from(value: Vec<v1::RolledDice>) -> Result<Self, Self::Error> {
        let encoded_rolled_dices =
            value.into_iter().map(RolledDice::try_from).collect::<Result<Vec<_>, _>>()?;

        Ok(Self(encoded_rolled_dices))
    }
}

impl From<RollDicesRequest> for v1::RollDicesRequest {
    fn from(value: RollDicesRequest) -> Self {
        let dices = value.dice_set.iter().map(|d| v1::DiceType::from(*d) as i32).collect();

        Self {
            dices,
        }
    }
}

impl TryFrom<v1::RollDicesRequest> for RollDicesRequest {
    type Error = Error;

    fn try_from(value: v1::RollDicesRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            dice_set: value
                .dices()
                .collect::<Vec<_>>()
                .try_into()
                .context("Cannot parse DiceSet")?,
        })
    }
}

impl From<RollDicesResponse> for v1::RollDicesResponse {
    fn from(value: RollDicesResponse) -> Self {
        Self {
            id: value.id.to_string(),
            rolled_dices: value.rolled_dice_set.into(),
        }
    }
}

impl TryFrom<v1::RollDicesResponse> for RollDicesResponse {
    type Error = Error;

    fn try_from(value: v1::RollDicesResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            id: RollId::parse(&value.id).context("Cannot parse UUID")?,
            rolled_dice_set: RolledDiceSet::try_from(value.rolled_dices)
                .context("Cannot parse the resulting dice set")?,
        })
    }
}

impl From<RollDicesResponse> for v1::GetDiceRollResponse {
    fn from(value: RollDicesResponse) -> Self {
        Self {
            id: value.id.to_string(),
            rolled_dices: value.rolled_dice_set.into(),
        }
    }
}

impl TryFrom<v1::GetDiceRollResponse> for RollDicesResponse {
    type Error = Error;

    fn try_from(value: v1::GetDiceRollResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            id: RollId::parse(&value.id).context("Cannot parse UUID")?,
            rolled_dice_set: RolledDiceSet::try_from(value.rolled_dices)
                .context("Cannot parse the resulting dice set")?,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Dice, DiceSet, RolledDice};

    #[test]
    fn can_encode_dice_protobuf() {
        let test_cases = &[
            (Dice::D3, v1::DiceType::DiceType3),
            (Dice::D4, v1::DiceType::DiceType4),
            (Dice::D6, v1::DiceType::DiceType6),
            (Dice::D8, v1::DiceType::DiceType8),
            (Dice::D10, v1::DiceType::DiceType10),
            (Dice::D12, v1::DiceType::DiceType12),
            (Dice::D20, v1::DiceType::DiceType20),
            (Dice::D100, v1::DiceType::DiceType100),
        ];

        for tc in test_cases {
            let proto_dice = v1::DiceType::from(tc.0);
            assert_eq!(proto_dice, tc.1);
        }
    }

    #[test]
    fn can_decode_dice_protobuf() {
        let test_cases = &[
            (v1::DiceType::DiceType3, Dice::D3),
            (v1::DiceType::DiceType4, Dice::D4),
            (v1::DiceType::DiceType6, Dice::D6),
            (v1::DiceType::DiceType8, Dice::D8),
            (v1::DiceType::DiceType10, Dice::D10),
            (v1::DiceType::DiceType12, Dice::D12),
            (v1::DiceType::DiceType20, Dice::D20),
            (v1::DiceType::DiceType100, Dice::D100),
        ];

        for tc in test_cases {
            let decoded_dice = Dice::try_from(tc.0).unwrap();
            assert_eq!(decoded_dice, tc.1);
        }

        let unspecified_dice = Dice::try_from(v1::DiceType::Unspecified);
        assert!(matches!(unspecified_dice, Err(crate::Error::Underlying(_))));
    }

    #[test]
    fn can_encode_rolled_dice_protobuf() {
        let rolled_dice = Dice::D100.roll();
        let proto_rolled_dice = v1::RolledDice::from(rolled_dice);
        assert_eq!(proto_rolled_dice.dice(), v1::DiceType::DiceType100);
        assert_eq!(proto_rolled_dice.result, rolled_dice.result);
    }

    #[test]
    fn can_decode_rolled_dice_protobuf() {
        let proto_rolled_dice = v1::RolledDice {
            dice: v1::DiceType::DiceType20 as i32,
            result: 19u32,
        };

        let decoded_rolled_dice = RolledDice::try_from(proto_rolled_dice).unwrap();

        assert_eq!(decoded_rolled_dice.dice, Dice::D20);
        assert_eq!(decoded_rolled_dice.result, 19);
    }

    #[test]
    fn can_encode_and_decode_dice_roll_requests() {
        let dice_set = DiceSet::new(vec![Dice::D100].into_iter());
        let req = RollDicesRequest {
            dice_set: dice_set.clone(),
        };

        let proto_req = v1::RollDicesRequest::from(req);

        assert_eq!(proto_req.dices.len(), 1);
        assert!(proto_req.dices().all(|d| d == v1::DiceType::DiceType100));

        let initial_req = RollDicesRequest::try_from(proto_req);
        assert!(initial_req.is_ok());

        assert_eq!(initial_req.unwrap().dice_set, dice_set);
    }

    #[tokio::test]
    async fn can_encode_and_decode_dice_roll_response() {
        let roll_dice_resp = RollDicesResponse {
            id: RollId::new(),
            rolled_dice_set: RolledDiceSet(vec![RolledDice {
                dice: Dice::D100,
                result: 98,
            }]),
        };

        let proto_roll_resp = v1::RollDicesResponse::from(roll_dice_resp.clone());

        assert_eq!(proto_roll_resp.rolled_dices.len(), 1);
        assert_eq!(proto_roll_resp.rolled_dices[0].dice(), v1::DiceType::DiceType100);
        assert_eq!(roll_dice_resp.id.into_string(), proto_roll_resp.id);
        assert_eq!(proto_roll_resp.rolled_dices[0].result, 98);
    }
}
