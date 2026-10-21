mod factory;
mod residential;
mod shop;

pub(super) use factory::factory_scene;
pub(super) use residential::residential_scene;
pub(super) use shop::shop_scene;

use crate::level::Interactable;
use avian2d::prelude::*;
use bevy::prelude::*;

fn building_scene(position: Vec2, size: Vec2, color: Color) -> impl Scene {
    bsn! {
        Interactable
        Sprite {
            color,
            custom_size: Option::Some(size),
        }
        // The whole placeholder is a sensor, so players can walk in front of it.
        Transform::from_xyz(position.x, position.y, -1.0)
        Collider::rectangle(size.x, size.y)
        Sensor
        CollidingEntities::default()
    }
}
