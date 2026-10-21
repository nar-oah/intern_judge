use crate::level::{InteractRequested, Interactable, SwitchPoint};
use avian2d::prelude::*;
use bevy::prelude::*;

pub(super) fn switch_point_scene(id: &'static str, position: Vec2) -> impl Scene {
    let size = Vec2::new(24.0, 50.0);
    bsn! {
        SwitchPoint { id }
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

fn interact(request: On<InteractRequested>, switch_points: Query<&SwitchPoint>) {
    if let Ok(point) = switch_points.get(request.entity) {
        info!(player = ?request.player, "Switch point triggered: {}", point.id);
    }
}
