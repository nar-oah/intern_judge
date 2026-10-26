use super::building_scene;
use crate::game::Money;
use crate::level::{BuildingType, InteractRequested};
use bevy::prelude::*;

const COST: i32 = 10;

pub(in crate::level::scene) fn lottery_store_scene(position: Vec2, size: Vec2) -> impl Scene {
    bsn! {
        @building_scene(position, size, Color::srgb(0.65, 0.5, 0.75))
        BuildingType::LotteryStore
        on(interact)
    }
}

fn interact(_: On<InteractRequested>, mut money: ResMut<Money>) {
    if money.0 < COST {
        return;
    }
    money.0 -= COST;
    money.0 += prize(rand::random_range(0..10));
}

fn prize(roll: u8) -> i32 {
    match roll {
        0 => 50,
        1..=2 => 10,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ten_equal_chances_have_one_jackpot_two_refunds_and_seven_losses() {
        let prizes: Vec<_> = (0..10).map(prize).collect();
        assert_eq!(prizes.iter().filter(|&&prize| prize == 50).count(), 1);
        assert_eq!(prizes.iter().filter(|&&prize| prize == 10).count(), 2);
        assert_eq!(prizes.iter().filter(|&&prize| prize == 0).count(), 7);
    }
}
