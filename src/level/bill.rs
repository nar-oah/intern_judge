mod ui;

use super::{
    SceneRoot,
    day::begin_next_day,
    scene,
    ui::{ConfirmArmed, UiAction},
};
use crate::{
    config::GameSettings,
    game::{CommuteState, Money},
};
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

#[derive(Component, Clone, Default)]
pub(super) struct WeeklyBill {
    total: i32,
}

pub(super) fn show_bill(world: &mut World) {
    if world
        .query_filtered::<Entity, With<WeeklyBill>>()
        .iter(world)
        .next()
        .is_some()
    {
        return;
    }
    let settings = world.resource::<GameSettings>();
    let balance = world.resource::<Money>().0;
    let salary = settings.weekly_salary;
    let rent = settings.weekly_rent;
    let bill = WeeklyBill {
        total: balance + salary - rent,
    };
    let next = ui::bill_scene(balance, salary, rent, bill);
    let roots = world
        .query_filtered::<Entity, With<SceneRoot>>()
        .iter(world)
        .collect::<Vec<_>>();
    scene::replace_scene(world, roots, next);
    world
        .resource_mut::<NextState<CommuteState>>()
        .set(CommuteState::Paused);
}

pub(super) fn confirm_bill(world: &mut World) {
    let mut bills = world.query::<(&WeeklyBill, &ActionState<UiAction>, &mut ConfirmArmed)>();
    let Ok((bill, actions, mut armed)) = bills.single_mut(world) else {
        return;
    };
    if !armed.just_pressed(actions) {
        return;
    }
    let total = bill.total;
    world.resource_mut::<Money>().0 = total;
    begin_next_day(world);
}
