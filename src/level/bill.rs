use super::{
    SceneRoot,
    day::begin_next_day,
    ui::{UiAction, confirm_input_scene},
};
use crate::{
    config::GameSettings,
    game::{CommuteState, Money},
};
use bevy::{prelude::*, text::FontSourceTemplate};
use leafwing_input_manager::prelude::ActionState;

#[derive(Component, Clone, Default)]
pub(super) struct WeeklyBill;

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
    let next = bill_scene(
        world.resource::<Money>().0,
        settings.weekly_salary,
        settings.weekly_rent,
    );
    let roots = world
        .query_filtered::<Entity, With<SceneRoot>>()
        .iter(world)
        .collect::<Vec<_>>();
    for root in roots {
        world.despawn(root);
    }
    world.spawn_scene(next).unwrap();
    world
        .resource_mut::<NextState<CommuteState>>()
        .set(CommuteState::Paused);
}

pub(super) fn confirm_bill(world: &mut World) {
    let mut bills = world.query_filtered::<&ActionState<UiAction>, With<WeeklyBill>>();
    let Ok(actions) = bills.single(world) else {
        return;
    };
    if !actions.just_pressed(&UiAction::Confirm) {
        return;
    }
    let settings = world.resource::<GameSettings>();
    let payment = settings.weekly_salary - settings.weekly_rent;
    world.resource_mut::<Money>().0 += payment;
    // Advancing the date also removes the bill, so this week cannot settle again.
    begin_next_day(world);
}

fn bill_scene(balance: i32, salary: i32, rent: i32) -> impl Scene {
    let summary = format!(
        "周五账单\n当前余额：{balance}\n本周工资：{salary}\n房租：{rent}\n结算后余额：{}\n按 E 确认",
        balance + salary - rent,
    );
    bsn! {
        #WeeklyBill
        SceneRoot
        WeeklyBill
        @confirm_input_scene()
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        }
        GlobalZIndex(300)
        BackgroundColor(Color::srgb(0.04, 0.05, 0.08))
        Pickable::default()
        Children [
            Text::new(summary)
            TextFont {
                font: FontSourceTemplate::Handle("fonts/court-dialogue.ttf"),
                font_size: FontSize::Px(28.0),
            }
            TextColor(Color::srgb(0.95, 0.96, 1.0))
            Pickable::IGNORE
        ]
    }
}
