use super::*;
use crate::config::LotteryPrize;

#[test]
fn configured_weights_define_each_prizes_chances() {
    let lottery = LotteryValues {
        price: 13,
        prizes: vec![
            LotteryPrize {
                reward: 99,
                weight: 0,
            },
            LotteryPrize {
                reward: 70,
                weight: 2,
            },
            LotteryPrize {
                reward: 13,
                weight: 3,
            },
            LotteryPrize {
                reward: 0,
                weight: 1,
            },
        ],
    };
    let prizes: Vec<_> = (0..6).map(|roll| prize(&lottery, roll)).collect();
    assert_eq!(prizes, [70, 70, 13, 13, 13, 0]);
}
