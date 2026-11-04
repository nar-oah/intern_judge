use super::{InitialRoad, RoadEffects, SceneRoot, bill, scene};
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
    let finished = calendar.is_finished();
    world.insert_resource(CommuteTime::default());
    world.insert_resource(Stamina::default());

    let next = if finished {
        Box::new(scene::ending::internship_ended_scene()) as Box<dyn Scene>
    } else {
        let road = world.resource::<InitialRoad>().0;
        let effects = world.resource::<RoadEffects>().0.get(&road);
        scene::road_scene(road, effects.map_or(&[], Vec::as_slice))
    };
    let roots = world
        .query_filtered::<Entity, With<SceneRoot>>()
        .iter(world)
        .collect::<Vec<_>>();
    if let Some((&root, others)) = roots.split_first() {
        for &other in others {
            world.despawn(other);
        }
        scene::replace_scene(world, root, next);
    } else {
        world.spawn_scene(next).unwrap();
    }
    world
        .resource_mut::<NextState<CommuteState>>()
        .set(if finished {
            CommuteState::Paused
        } else {
            CommuteState::Running
        });
}
