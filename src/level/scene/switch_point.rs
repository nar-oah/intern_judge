use crate::level::{InteractRequested, Interactable, SwitchPoint, navigation};
use avian2d::prelude::*;
use bevy::prelude::*;

pub(super) fn switch_point_scene(position: Vec2) -> impl Scene {
    let size = Vec2::new(24.0, 50.0);
    bsn! {
        SwitchPoint
        Interactable
        Sprite {
            color: Color::srgb(0.3, 0.85, 0.85),
            custom_size: Option::Some(size),
        }
        Transform::from_xyz(position.x, position.y, -1.0)
        Collider::rectangle(size.x, size.y)
        Sensor
        CollidingEntities::default()
        on(interact)
    }
}

fn interact(request: On<InteractRequested>, mut commands: Commands) {
    let point = request.entity;
    let player = request.player;
    commands.queue(move |world: &mut World| navigation::open_destinations(world, point, player));
}
