mod building;
mod interaction;
mod platform;
mod road;
mod transition;

#[cfg(test)]
mod tests;

use bevy::prelude::*;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<interaction::DemoWallet>()
            .add_systems(Startup, road::road_01.spawn())
            .add_systems(Update, interaction::interact);
    }
}
