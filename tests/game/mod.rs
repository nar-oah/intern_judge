use super::calendar::{CalendarHud, CalendarHudText};
use super::*;
use crate::test_support::{advance, finish_app, insert_config, count, single, text};
use bevy::{state::app::StatesPlugin, time::TimeUpdateStrategy};
use std::time::Duration;


mod calendar;
mod commute;
mod hud;
mod session;
mod support;
use support::game;
