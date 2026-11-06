use super::{ui::DialogueChoiceButton, *};
use crate::{
    config::{CaseDefs, GameState},
    game::{CommuteState, GamePlugin, Money, Stamina, calendar::GameCalendar},
    level::{
        CurrentCase, InitialRoad, Interactable, LevelPlugin, PausedInteractRequested, RoadEffect,
        RoadEffects, RoadId, SceneRoot, SelectedCase, SwitchPoint,
        bill::WeeklyBill,
        scene::ending::InternshipEnded,
        tests::support::{open_destinations, select_destination},
    },
    player::{Player, PlayerPlugin},
    test_support::{
        advance, assert_absent, case_def, finish_app, headless_app, named, press_e, press_key,
        single, text,
    },
};
use avian2d::prelude::{Collider, CollidingEntities, LinearVelocity, Position, Sensor, ShapeHits};
use bevy::{
    asset::io::embedded::EmbeddedAssetRegistry,
    ui_widgets::{Activate, Button},
};
use leafwing_input_manager::prelude::*;
use std::{
    collections::HashMap,
    path::Path,
    time::{Duration, Instant},
};

mod controls;
mod court;
mod progression;
mod support;
mod verdicts;

use support::*;
