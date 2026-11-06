use super::{CommuteState, CommuteTime, DailyTransactions, HasHelmet, Money, Stamina};
use crate::config::GameSettings;
use bevy::prelude::*;

pub(super) fn initialize_game(
    mut commands: Commands,
    settings: Res<GameSettings>,
    mut next_commute: ResMut<NextState<CommuteState>>,
) {
    commands.insert_resource(CommuteTime::default());
    commands.insert_resource(Money(settings.initial_money));
    commands.insert_resource(DailyTransactions::default());
    commands.insert_resource(Stamina::default());
    commands.insert_resource(HasHelmet::default());
    next_commute.set(CommuteState::Running);
}
