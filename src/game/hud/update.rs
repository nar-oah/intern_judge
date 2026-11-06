use super::{
    calendar::CalendarHudText,
    case::{CaseHud, CaseHudText},
    commute::{CommuteHudText, StaminaBar},
};
use crate::{
    config::GameSettings,
    game::{CommuteTime, DailyTransactions, Money, Stamina, calendar::GameCalendar},
    level::CurrentCase,
};
use bevy::prelude::*;

pub(super) fn update_commute_hud(
    settings: Res<GameSettings>,
    commute: Res<CommuteTime>,
    money: Res<Money>,
    transactions: Res<DailyTransactions>,
    stamina: Res<Stamina>,
    mut hud: Query<&mut Text, With<CommuteHudText>>,
    mut bars: Query<&mut Node, With<StaminaBar>>,
) {
    for mut text in &mut hud {
        text.0 = format!(
            "Time: {:.1}s\nMoney: {}\nStamina: {:.0}/{:.0}",
            commute.remaining(settings.commute_limit),
            money.0 + transactions.net(),
            stamina.0,
            Stamina::MAX,
        );
    }
    for mut node in &mut bars {
        node.width = percent((stamina.0 / Stamina::MAX).clamp(0.0, 1.0) * 100.0);
    }
}

pub(super) fn update_calendar_hud(
    calendar: Res<GameCalendar>,
    mut hud: Query<&mut Text, With<CalendarHudText>>,
) {
    let weekdays = [
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
        "Sunday",
    ];
    for mut text in &mut hud {
        text.0 = format!(
            "Days left: {}\n{}{}",
            calendar.remaining_days(),
            weekdays[(calendar.weekday() - 1) as usize],
            if calendar.is_weekend() {
                " (Weekend)"
            } else {
                ""
            },
        );
    }
}

pub(super) fn update_case_hud(
    current_case: Res<CurrentCase>,
    mut roots: Query<&mut Visibility, With<CaseHud>>,
    mut labels: Query<&mut Text, With<CaseHudText>>,
) {
    let road = current_case.0.as_ref().map(|selected| selected.road);
    for mut visibility in &mut roots {
        *visibility = if road.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for mut text in &mut labels {
        text.0 = road.map_or_else(String::new, |road| format!("今日涉案路段：{road:?}"));
    }
}
