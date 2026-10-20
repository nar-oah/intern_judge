use super::{BuildingType, Interactable, Platform, RoadId, SwitchPoint};
use avian2d::prelude::*;
use bevy::prelude::*;

pub(super) fn platform_scene(position: Vec2, size: Vec2) -> impl Scene {
    let platform = Platform { size };
    bsn! {
        platform
        Sprite {
            color: Color::srgb(0.65, 0.68, 0.75),
            custom_size: Option::Some(platform.size),
        }
        Transform::from_xyz(position.x, position.y, 0.0)
        RigidBody::Static
        Collider::rectangle(platform.size.x, platform.size.y)
    }
}

fn building_scene(position: Vec2, size: Vec2, color: Color) -> impl Scene {
    bsn! {
        Interactable { radius: 75.0 }
        Sprite {
            color,
            custom_size: Option::Some(size),
        }
        // Walk in front of the placeholder; buildings do not block the road.
        Transform::from_xyz(position.x, position.y, -1.0)
    }
}

pub(super) fn residential_scene(position: Vec2, size: Vec2) -> impl Scene {
    bsn! {
        @building_scene(position, size, Color::srgb(0.45, 0.7, 0.5))
        BuildingType::Residential
    }
}

pub(super) fn shop_scene(position: Vec2, size: Vec2) -> impl Scene {
    bsn! {
        @building_scene(position, size, Color::srgb(0.9, 0.65, 0.3))
        BuildingType::Shop
    }
}

pub(super) fn factory_scene(position: Vec2, size: Vec2) -> impl Scene {
    bsn! {
        @building_scene(position, size, Color::srgb(0.65, 0.5, 0.75))
        BuildingType::Factory
    }
}

pub(super) fn switch_point_scene(id: &'static str, position: Vec2) -> impl Scene {
    bsn! {
        SwitchPoint { id }
        Interactable { radius: 60.0 }
        Sprite {
            color: Color::srgb(0.3, 0.85, 0.85),
            custom_size: Option::Some(Vec2::new(24.0, 50.0)),
        }
        Transform::from_xyz(position.x, position.y, -1.0)
    }
}

/// Positions are local to this root. Despawning it also removes the whole road.
pub(super) fn example_road_scene() -> impl Scene {
    bsn! {
        #ExampleRoad
        RoadId("example_road")
        Transform::default()
        Visibility::default()
        Children [
            @platform_scene(Vec2::new(0.0, -180.0), Vec2::new(700.0, 35.0))
            --
            @platform_scene(Vec2::new(-425.0, -180.0), Vec2::new(150.0, 35.0))
            --
            @platform_scene(Vec2::new(425.0, -180.0), Vec2::new(150.0, 35.0))
            --
            @platform_scene(Vec2::new(-170.0, -105.0), Vec2::new(130.0, 20.0))
            --
            @platform_scene(Vec2::new(170.0, -90.0), Vec2::new(150.0, 20.0))
            --
            @residential_scene(Vec2::new(-310.0, -122.5), Vec2::new(65.0, 80.0))
            --
            @shop_scene(Vec2::new(0.0, -127.5), Vec2::new(70.0, 70.0))
            --
            @factory_scene(Vec2::new(310.0, -117.5), Vec2::new(90.0, 90.0))
            --
            @switch_point_scene("west_exit", Vec2::new(-460.0, -137.5))
            --
            @switch_point_scene("east_exit", Vec2::new(460.0, -137.5))
        ]
    }
}
