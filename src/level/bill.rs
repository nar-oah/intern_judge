mod ui;

use super::{
    SceneRoot,
    day::{EndDay, start_day},
    ui::{ConfirmArmed, UiAction},
};
use crate::{
    config::GameSettings,
    game::{CommuteState, CommuteTime, DailyTransactions, Money, Stamina, calendar::GameCalendar},
};
use bevy::{ecs::system::SystemParam, prelude::*};
use leafwing_input_manager::prelude::ActionState;

#[derive(Component, Clone, Default)]
pub(super) struct DailyBill {
    balance: i32,
    salary: i32,
    rent: i32,
    transactions: DailyTransactions,
    total: i32,
}

#[derive(SystemParam)]
pub(super) struct BillResources<'w> {
    calendar: Res<'w, GameCalendar>,
    time: Res<'w, CommuteTime>,
    settings: Res<'w, GameSettings>,
    money: Res<'w, Money>,
    transactions: Res<'w, DailyTransactions>,
}

pub(super) fn show_bill(
    _: On<EndDay>,
    mut commands: Commands,
    roots: Query<Entity, With<SceneRoot>>,
    resources: BillResources,
    mut commute: ResMut<NextState<CommuteState>>,
) {
    let BillResources {
        calendar,
        time,
        settings,
        money,
        transactions,
    } = resources;
    let remaining = time.remaining(settings.commute_limit).max(0.0);
    let multiplier = remaining / settings.work_time as f32;
    let multiplier = if remaining == 0.0 { multiplier } else { 0.0 };
    let salary = (settings.base_salary as f32 * multiplier).round() as i32;
    let rent = settings.weekly_rent;
    let rent = if calendar.weekday() == 5 { rent } else { 0 };
    let bill = DailyBill {
        balance: money.0,
        salary,
        rent,
        transactions: *transactions,
        total: money.0 + transactions.net() + salary - rent,
    };
    commands.spawn_scene(ui::bill_scene(bill));
    for root in &roots {
        commands.entity(root).despawn();
    }
    commute.set(CommuteState::Paused);
}

pub(super) fn confirm_bill(
    mut commands: Commands,
    mut bills: Query<(&DailyBill, &ActionState<UiAction>, &mut ConfirmArmed)>,
    mut money: ResMut<Money>,
    mut transactions: ResMut<DailyTransactions>,
    mut calendar: ResMut<GameCalendar>,
    mut time: ResMut<CommuteTime>,
    mut stamina: ResMut<Stamina>,
) {
    let Ok((bill, actions, mut armed)) = bills.single_mut() else {
        return;
    };
    if !armed.just_pressed(actions) {
        return;
    }
    money.0 = bill.total;
    *transactions = DailyTransactions::default();
    calendar.elapsed_days += 1;
    *time = CommuteTime::default();
    *stamina = Stamina::default();
    commands.run_system_cached(start_day);
}
