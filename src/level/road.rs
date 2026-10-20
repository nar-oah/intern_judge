use bevy::prelude::*;

use super::{
    building::{convenience_store, restaurant},
    platform::platform,
    transition::transition_point,
};

// This ID will identify a road in the eventual route / verdict systems.
// Entity IDs are intentionally not used as persistent road IDs.
#[derive(Component, Clone, Default)]
pub(super) struct RoadId(pub u8);

// This file is the entire level layout: what to place, where, and how large.
// Positions are local to the road root and can be changed independently.
pub(super) fn road_01() -> impl Scene {
    bsn! {
        #Road01
        RoadId(1)
        Children [
            @platform(Vec2::new(700.0, 35.0))
            Transform::from_xyz(0.0, -180.0, 0.0)
            --
            @platform(Vec2::new(150.0, 20.0))
            Transform::from_xyz(170.0, -90.0, 0.0)
            --
            @restaurant()
            Transform::from_xyz(-220.0, -117.5, 1.0)
            --
            @convenience_store()
            Transform::from_xyz(-70.0, -117.5, 1.0)
            --
            @transition_point()
            Transform::from_xyz(-320.0, -127.5, 1.0)
            --
            @transition_point()
            Transform::from_xyz(310.0, -127.5, 1.0)
        ]
    }
}
