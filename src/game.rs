pub mod calendar;
mod commute;
mod hud;
mod session;
#[cfg(test)]
#[path = "../tests/game/mod.rs"]
mod tests;
use crate::config::{GameSettings, GameState};
use bevy::prelude::*;

#[derive(Resource, Debug, Default)]
pub struct CommuteTime {
    pub elapsed: f32,
}

impl CommuteTime {
    pub fn remaining(&self, limit: f32) -> f32 {
        limit - self.elapsed
    }
}

#[derive(Resource, Debug)]
pub struct Money(pub i32);

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DailyTransactions {
    pub convenience_store_cost: i32,
    pub tool_store_cost: i32,
    pub cat_reward: i32,
    pub lottery_cost: i32,
    pub lottery_reward: i32,
}

impl DailyTransactions {
    pub fn net(&self) -> i32 {
        self.cat_reward + self.lottery_reward
            - self.convenience_store_cost
            - self.tool_store_cost
            - self.lottery_cost
    }
}

impl FromWorld for Money {
    fn from_world(world: &mut World) -> Self {
        Self(world.resource::<GameSettings>().initial_money)
    }
}

#[derive(Resource, Debug, Default)]
pub struct HasHelmet(pub bool);

#[derive(Resource, Debug)]
pub struct Stamina(pub f32);

impl Stamina {
    pub const MAX: f32 = 100.0;
}

impl Default for Stamina {
    fn default() -> Self {
        Self(Self::MAX)
    }
}

#[derive(States, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CommuteState {
    #[default]
    Running,
    Paused,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<CommuteState>()
            .add_systems(
                OnEnter(GameState::Playing),
                (session::initialize_game, calendar::initialize_calendar).chain(),
            )
            .add_systems(
                Update,
                commute::tick_commute
                    .run_if(in_state(CommuteState::Running))
                    .run_if(in_state(GameState::Playing)),
            );
        hud::setup(app);
    }
}
