//! Shared headless apps, deterministic data, ECS assertions and keyboard input.
mod app;
mod ecs;
mod fixtures;
mod input;

pub(crate) use app::{advance, finish_app, headless_app, level_app, scene_app};
pub(crate) use ecs::{assert_absent, assert_removed, count, named, scene_entities, single, text};
pub(crate) use fixtures::{building_values, case_def, game_settings, insert_config};
pub(crate) use input::{press_e, press_key};
