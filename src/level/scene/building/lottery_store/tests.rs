use super::*;
use crate::config::LotteryPrize;

#[test]
fn prize_intervals_include_their_first_roll_and_exclude_the_next_interval() {
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
                reward: 99,
                weight: 0,
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

    for (roll, expected_reward) in [(0, 70), (1, 70), (2, 13), (4, 13), (5, 0)] {
        assert_eq!(prize(&lottery, roll), expected_reward, "roll {roll}");
    }
}
