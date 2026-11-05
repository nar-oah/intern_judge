mod cat;
mod convenience_store;
mod lottery_store;
#[cfg(test)]
mod tests;
mod tool_store;
use crate::{
    level::{Interactable, PausedInteractRequested, dialogue::ui::dialogue_box_scene},
    player::Player,
};
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_replique::prelude::{DialogueRunner, StartDialogue};
pub(super) use cat::cat_scene;
pub(super) use convenience_store::convenience_store_scene;
pub(super) use lottery_store::lottery_store_scene;
pub(super) use tool_store::tool_store_scene;

#[derive(Component, Clone)]
pub(in crate::level::scene) struct BuildingDialogue {
    asset: &'static str,
    node: String,
}

impl BuildingDialogue {
    pub(in crate::level::scene) fn new(asset: &'static str, node: impl Into<String>) -> Self {
        Self { asset, node: node.into() }
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
    request: On<PausedInteractRequested>,
    buildings: Query<&BuildingDialogue>,
    assets: Res<AssetServer>,
    mut commands: Commands,
    mut players: Query<&mut LinearVelocity, (With<Player>, Without<DialogueRunner>)>,
    mut start: MessageWriter<StartDialogue>,
) {
    let Ok(dialogue) = buildings.get(request.entity) else {
        return;
    };
    let Ok(mut velocity) = players.get_mut(request.player) else {
        return;
    };
    velocity.x = 0.0;
    commands
        .entity(request.player)
        .insert(DialogueRunner::new(assets.load(dialogue.asset)));
    commands.spawn_scene(dialogue_box_scene());
    start.write(StartDialogue {
        runner: request.player,
        node: dialogue.node.clone(),
    });
}
