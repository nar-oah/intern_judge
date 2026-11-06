use super::{CommuteState, CommuteTime, HasHelmet, Money, Stamina};
use bevy::prelude::*;

pub(super) fn initialize_game(world: &mut World) {
    world.insert_resource(CommuteTime::default());
    let money = Money::from_world(world);
    world.insert_resource(money);
    world.insert_resource(Stamina::default());
    world.insert_resource(HasHelmet::default());
    world
        .resource_mut::<NextState<CommuteState>>()
        .set(CommuteState::Running);
}
