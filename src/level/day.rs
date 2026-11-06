use super::{CurrentCase, InitialRoad, RoadId, SceneRoot, TodayRoads, navigation, scene};
use crate::game::{CommuteState, calendar::GameCalendar};
use bevy::prelude::*;

#[derive(Event)]
pub(super) struct EndDay;

pub(super) fn start_day(
    mut commands: Commands,
    roots: Query<Entity, With<SceneRoot>>,
    calendar: Res<GameCalendar>,
    initial: Res<InitialRoad>,
    mut commute: ResMut<NextState<CommuteState>>,
) {
    commands.insert_resource(TodayRoads::default());
    commands.insert_resource(CurrentCase::default());
    if calendar.is_finished() {
        for root in &roots {
            commands.entity(root).despawn();
        }
        commands.spawn_scene(scene::ending::internship_ended_scene());
        commute.set(CommuteState::Paused);
    } else if calendar.is_weekend() {
        commands.trigger(navigation::OpenUnlockMap);
    } else {
        commands.trigger(scene::EnterRoad(initial.0));
        commute.set(if initial.0 == RoadId::Court {
            CommuteState::Paused
        } else {
            CommuteState::Running
        });
    }
}
