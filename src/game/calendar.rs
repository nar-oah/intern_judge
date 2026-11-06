use crate::config::GameSettings;
use bevy::prelude::*;

#[derive(Resource, Debug)]
pub struct GameCalendar {
    pub total_days: u32,
    pub elapsed_days: u32,
}

impl GameCalendar {
    pub fn weekday(&self) -> u32 {
        (self.elapsed_days % 7 + 5) % 7 + 1
    }
    pub fn remaining_days(&self) -> u32 {
        self.total_days.saturating_sub(self.elapsed_days)
    }
    pub fn is_weekend(&self) -> bool {
        self.weekday() >= 6
    }
    pub fn is_finished(&self) -> bool {
        self.elapsed_days >= self.total_days
    }
}

pub(crate) fn initialize_calendar(mut commands: Commands, settings: Res<GameSettings>) {
    commands.insert_resource(GameCalendar {
        total_days: settings.total_days,
        elapsed_days: 0,
    });
}
