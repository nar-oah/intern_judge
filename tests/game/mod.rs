use super::hud::{
    calendar::{CalendarHud, CalendarHudText},
    case::{CaseHud, CaseHudText},
    commute::{CommuteHud, CommuteHudText, StaminaBar},
};
use super::*;
use super::calendar::GameCalendar;
use crate::{
    level::CurrentCase,
    test_support::{advance, count, finish_app, insert_config, scene_app, single, text},
};
use bevy::{state::app::StatesPlugin, time::TimeUpdateStrategy};
use std::time::Duration;

mod calendar;
mod commute;
mod hud;
mod session;

fn game() -> App {
    let mut app = scene_app();
    app.add_plugins(StatesPlugin);
    insert_config(&mut app);
    app.init_resource::<CurrentCase>();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        125,
    )))
    .add_plugins(GamePlugin);
    finish_app(&mut app);
    app
}
