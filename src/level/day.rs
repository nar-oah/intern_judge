use super::{CurrentCase, InitialRoad, RoadId, SceneRoot, TodayRoads, bill, navigation, scene};
use crate::game::{CommuteState, CommuteTime, Stamina, calendar::GameCalendar};
use bevy::prelude::*;

#[derive(Resource)]
pub(super) struct DayEndRequested;

pub(super) fn finish_day(world: &mut World) {
    world.remove_resource::<DayEndRequested>();
    let calendar = world.resource::<GameCalendar>();
    if !calendar.is_finished() && calendar.weekday() == 5 {
        bill::show_bill(world);
    } else {
        begin_next_day(world);
    }
}

pub(super) fn begin_next_day(world: &mut World) {
    world.remove_resource::<DayEndRequested>();
    let mut calendar = world.resource_mut::<GameCalendar>();
    if calendar.is_finished() {
        return;
    }
    calendar.elapsed_days += 1;
    world.insert_resource(CommuteTime::default());
    world.insert_resource(Stamina::default());
    start_day(world);
}

pub(super) fn start_day(world: &mut World) {
    world.insert_resource(TodayRoads::default());
    world.insert_resource(CurrentCase::default());
    let calendar = world.resource::<GameCalendar>();
    let finished = calendar.is_finished();
    if !finished && calendar.is_weekend() {
        world.trigger(navigation::OpenUnlockMap);
        world.flush();
        return;
    }
    let road = world.resource::<InitialRoad>().0;
    if finished {
        let roots = world
            .query_filtered::<Entity, With<SceneRoot>>()
            .iter(world)
            .collect::<Vec<_>>();
        scene::replace_scene(world, roots, scene::ending::internship_ended_scene());
    } else {
        world.trigger(scene::EnterRoad(road));
        // Replace the old scene before a caller continues the day transition.
        world.flush();
    }
    world
        .resource_mut::<NextState<CommuteState>>()
        .set(if finished || road == RoadId::Court {
            CommuteState::Paused
        } else {
            CommuteState::Running
        });
}
