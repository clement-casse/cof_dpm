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

use anyhow::{anyhow, Context as _};
use pb::dice_api::v1;

use crate::{
    service::{RollDicesRequest, RollDicesResponse}, DiceSet, Die, Error, RollId, RolledDiceSet,
    RolledDie,
};

impl From<Die> for v1::DiceType {
    fn from(value: Die) -> Self {
        match value {
            Die::D3 => Self::DiceType3,
            Die::D4 => Self::DiceType4,
            Die::D6 => Self::DiceType6,
            Die::D8 => Self::DiceType8,
            Die::D10 => Self::DiceType10,
            Die::D12 => Self::DiceType12,
            Die::D20 => Self::DiceType20,
            Die::D100 => Self::DiceType100,
        }
    }
}

impl TryFrom<v1::DiceType> for Die {
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
            v1::DiceType::Unspecified => Err(Error::Underlying(anyhow!(
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
        let encoded_dices = value
            .into_iter()
            .map(Die::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self(encoded_dices))
    }
}

impl From<RolledDie> for v1::RolledDice {
    fn from(value: RolledDie) -> Self {
        Self {
            dice: v1::DiceType::from(value.dice) as i32,
            result: value.result,
        }
    }
}

impl TryFrom<v1::RolledDice> for RolledDie {
    type Error = Error;

    fn try_from(value: v1::RolledDice) -> Result<Self, Self::Error> {
        let dice = Die::try_from(value.dice())?;
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
        let encoded_rolled_dices = value
            .into_iter()
            .map(RolledDie::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self(encoded_rolled_dices))
    }
}

impl From<RollDicesRequest> for v1::RollDicesRequest {
    fn from(value: RollDicesRequest) -> Self {
        let dices = value
            .dice_set
            .iter()
            .map(|d| v1::DiceType::from(*d) as i32)
            .collect();

        Self { dices }
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
    use crate::{DiceSet, Die, RolledDie};

    #[test]
    fn can_encode_dice_protobuf() {
        let test_cases = &[
            (Die::D3, v1::DiceType::DiceType3),
            (Die::D4, v1::DiceType::DiceType4),
            (Die::D6, v1::DiceType::DiceType6),
            (Die::D8, v1::DiceType::DiceType8),
            (Die::D10, v1::DiceType::DiceType10),
            (Die::D12, v1::DiceType::DiceType12),
            (Die::D20, v1::DiceType::DiceType20),
            (Die::D100, v1::DiceType::DiceType100),
        ];

        for tc in test_cases {
            let proto_dice = v1::DiceType::from(tc.0);
            assert_eq!(proto_dice, tc.1);
        }
    }

    #[test]
    fn can_decode_dice_protobuf() {
        let test_cases = &[
            (v1::DiceType::DiceType3, Die::D3),
            (v1::DiceType::DiceType4, Die::D4),
            (v1::DiceType::DiceType6, Die::D6),
            (v1::DiceType::DiceType8, Die::D8),
            (v1::DiceType::DiceType10, Die::D10),
            (v1::DiceType::DiceType12, Die::D12),
            (v1::DiceType::DiceType20, Die::D20),
            (v1::DiceType::DiceType100, Die::D100),
        ];

        for tc in test_cases {
            let decoded_dice = Die::try_from(tc.0).unwrap();
            assert_eq!(decoded_dice, tc.1);
        }

        let unspecified_dice = Die::try_from(v1::DiceType::Unspecified);
        assert!(matches!(unspecified_dice, Err(Error::Underlying(_))));
    }

    #[test]
    fn can_encode_rolled_dice_protobuf() {
        let rolled_dice = Die::D100.roll();
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

        let decoded_rolled_dice = RolledDie::try_from(proto_rolled_dice).unwrap();

        assert_eq!(decoded_rolled_dice.dice, Die::D20);
        assert_eq!(decoded_rolled_dice.result, 19);
    }

    #[test]
    fn can_encode_and_decode_dice_roll_requests() {
        let dice_set = DiceSet::new(vec![Die::D100].into_iter());
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
            rolled_dice_set: RolledDiceSet(vec![RolledDie {
                dice: Die::D100,
                result: 98,
            }]),
        };

        let proto_roll_resp = v1::RollDicesResponse::from(roll_dice_resp.clone());

        assert_eq!(proto_roll_resp.rolled_dices.len(), 1);
        assert_eq!(
            proto_roll_resp.rolled_dices[0].dice(),
            v1::DiceType::DiceType100
        );
        assert_eq!(roll_dice_resp.id.into_string(), proto_roll_resp.id);
        assert_eq!(proto_roll_resp.rolled_dices[0].result, 98);
    }
}
