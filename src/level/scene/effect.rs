mod construction_detour;
mod falling_object;
#[cfg(test)]
mod tests;

use bevy::prelude::*;
pub(super) use construction_detour::construction_detour_scene;
pub(super) use falling_object::falling_object_scene;

#[derive(Component, Clone, Default)]
pub(crate) struct FallingObject;

#[derive(Component, Clone, Default)]
pub(crate) struct ConstructionDetour;
