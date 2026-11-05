use super::calendar::{CalendarHud, CalendarHudText};
use super::*;
use crate::test_support::{advance, finish_app, insert_config, count, single, text};
use bevy::{state::app::StatesPlugin, time::TimeUpdateStrategy};
use std::time::Duration;


mod calendar;
mod commute;
mod hud;
mod session;

fn game() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin));
    insert_config(&mut app);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        125,
    )))
    .add_plugins(GamePlugin);
    finish_app(&mut app);
    app
}

