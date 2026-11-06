pub(super) mod calendar;
pub(super) mod case;
pub(super) mod commute;
mod update;

use super::{CommuteTime, Money, Stamina, calendar::GameCalendar};
use crate::{
    config::{GameSettings, GameState},
    level::CurrentCase,
};
use bevy::{camera::visibility::VisibilitySystems, prelude::*, ui::UiSystems};
use bevy_replique::prelude::DialogueSystem;
use calendar::CalendarHud;
use case::CaseHud;
use commute::CommuteHud;

pub(super) fn setup(app: &mut App) {
    app.add_systems(
        OnEnter(GameState::Playing),
        setup_hud.after(super::calendar::initialize_calendar),
    )
    .add_systems(OnExit(GameState::Playing), cleanup_hud)
    .add_systems(
        Update,
        (
            update::update_commute_hud.run_if(
                resource_exists_and_changed::<CommuteTime>
                    .or_eager(resource_exists_and_changed::<Money>)
                    .or_eager(resource_exists_and_changed::<Stamina>)
                    .or_eager(resource_exists_and_changed::<GameSettings>),
            ),
            update::update_calendar_hud.run_if(resource_exists_and_changed::<GameCalendar>),
        )
            .chain()
            .after(super::commute::tick_commute)
            .run_if(in_state(GameState::Playing)),
    )
    .add_systems(
        PostUpdate,
        update::update_case_hud
            .after(DialogueSystem::Commands)
            .before(UiSystems::Prepare)
            .before(VisibilitySystems::VisibilityPropagate)
            .run_if(resource_exists_and_changed::<CurrentCase>)
            .run_if(in_state(GameState::Playing)),
    );
}

fn setup_hud(mut commands: Commands, current_case: Option<Res<CurrentCase>>) {
    commands.spawn_scene(commute::commute_hud_scene());
    commands.spawn_scene(calendar::calendar_hud_scene());
    let road = current_case
        .as_deref()
        .and_then(|current| current.0.as_ref())
        .map(|selected| selected.road);
    commands.spawn_scene(case::case_hud_scene(road));
}

fn cleanup_hud(
    mut commands: Commands,
    roots: Query<Entity, Or<(With<CommuteHud>, With<CalendarHud>, With<CaseHud>)>>,
) {
    for root in &roots {
        commands.entity(root).despawn();
    }
}
