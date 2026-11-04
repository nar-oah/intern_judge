mod cat;
mod convenience_store;
mod lottery_store;
#[cfg(test)]
mod tests;
mod tool_store;
use crate::{
    game::{CommuteState, calendar::GameCalendar},
    level::{InteractRequested, Interactable, dialogue::ui::dialogue_box_scene},
    player::Player,
};
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_replique::prelude::{DialogueRunner, StartDialogue};
pub(super) use cat::cat_scene;
pub(super) use convenience_store::convenience_store_scene;
pub(super) use lottery_store::lottery_store_scene;
pub(super) use tool_store::tool_store_scene;

#[derive(Component, Clone, Copy)]
pub(in crate::level::scene) struct BuildingDialogue {
    asset: &'static str,
    node: &'static str,
    weekend_node: Option<&'static str>,
    paused_only: bool,
}

impl BuildingDialogue {
    pub(in crate::level::scene) fn new(asset: &'static str, node: &'static str) -> Self {
        Self {
            asset,
            node,
            weekend_node: None,
            paused_only: false,
        }
    }

    pub(in crate::level::scene) fn with_weekend_node(mut self, node: &'static str) -> Self {
        self.weekend_node = Some(node);
        self
    }

    fn when_paused(mut self) -> Self {
        self.paused_only = true;
        self
    }
}

pub(in crate::level::scene) fn dialogue_building_scene(
    position: Vec2,
    size: Vec2,
    color: Color,
    dialogue: BuildingDialogue,
) -> impl Scene {
    bsn! {
        @building_scene(position, size, color)
        dialogue
        on(start_building_dialogue)
    }
}

fn building_scene(position: Vec2, size: Vec2, color: Color) -> impl Scene {
    bsn! {
        Interactable
        Sprite {
            color,
            custom_size: Option::Some(size),
        }
        Transform::from_xyz(position.x, position.y, -1.0)
        Collider::rectangle(size.x, size.y)
        Sensor
        CollidingEntities::default()
    }
}

fn start_building_dialogue(
    request: On<InteractRequested>,
    buildings: Query<&BuildingDialogue>,
    commute: Res<State<CommuteState>>,
    calendar: Res<GameCalendar>,
    assets: Res<AssetServer>,
    mut commands: Commands,
    mut players: Query<&mut LinearVelocity, (With<Player>, Without<DialogueRunner>)>,
    mut start: MessageWriter<StartDialogue>,
) {
    let Ok(dialogue) = buildings.get(request.entity) else {
        return;
    };
    if dialogue.paused_only && *commute.get() != CommuteState::Paused {
        return;
    }
    let Ok(mut velocity) = players.get_mut(request.player) else {
        return;
    };
    let node = if calendar.is_weekend() {
        dialogue.weekend_node.unwrap_or(dialogue.node)
    } else {
        dialogue.node
    };
    velocity.x = 0.0;
    commands
        .entity(request.player)
        .insert(DialogueRunner::new(assets.load(dialogue.asset)));
    commands.spawn_scene(dialogue_box_scene());
    start.write(StartDialogue {
        runner: request.player,
        node: node.into(),
    });
}
