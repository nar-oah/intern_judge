use super::FallingObject;
use crate::{
    game::HasHelmet,
    player::{Player, Stunned},
};
use avian2d::prelude::*;
use bevy::prelude::*;

pub(in crate::level::scene) fn falling_object_scene(position: Vec2) -> impl Scene {
    let size = Vec2::splat(30.0);
    bsn! {
        FallingObject
        Sprite {
            color: Color::srgb(0.85, 0.25, 0.35),
            custom_size: Option::Some(size),
        }
        Transform::from_translation(position.extend(1.0))
        RigidBody::Dynamic
        Collider::rectangle(size.x, size.y)
        CollisionEventsEnabled
        on(collide)
    }
}

fn collide(event: On<CollisionStart>, mut commands: Commands) {
    let object = event.collider1;
    let target = event.collider2;
    commands.queue(move |world: &mut World| {
        // A single physics step can report several contacts for the same object.
        if world.get::<FallingObject>(object).is_none() {
            return;
        }
        world.despawn(object);
        if world.get::<Player>(target).is_none() {
            return;
        }
        let mut helmet = world.resource_mut::<HasHelmet>();
        if helmet.0 {
            helmet.0 = false;
            return;
        }
        let mut player = world.entity_mut(target);
        if let Some(mut velocity) = player.get_mut::<LinearVelocity>() {
            velocity.x = 0.0;
        }
        player.insert(Stunned { remaining: 2.0 });
    });
}
