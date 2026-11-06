use crate::level::{
    Interactable, PausedInteractRequested, RunningInteractRequested, SwitchPoint,
    day::DayEndRequested, navigation,
};
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
        on(navigation::open_destinations)
        on(end_day)
    }
}

fn end_day(_: On<PausedInteractRequested>, mut commands: Commands) {
    commands.insert_resource(DayEndRequested);
}
