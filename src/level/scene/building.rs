mod cat;
mod convenience_store;
mod lottery_store;
#[cfg(test)]
mod tests;
mod tool_store;
use crate::level::Interactable;
use avian2d::prelude::*;
use bevy::prelude::*;
pub(super) use cat::cat_scene;
pub(super) use convenience_store::convenience_store_scene;
pub(super) use lottery_store::lottery_store_scene;
pub(super) use tool_store::tool_store_scene;

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
