#[cfg(test)]
mod tests;

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub(in crate::level) enum UiAction {
    Confirm,
}

#[derive(Component, Clone, Default)]
pub(in crate::level) struct ConfirmArmed(bool);

impl ConfirmArmed {
    pub(in crate::level) fn just_pressed(&mut self, actions: &ActionState<UiAction>) -> bool {
        if !actions.pressed(&UiAction::Confirm) {
            self.0 = true;
        }
        self.0 && actions.just_pressed(&UiAction::Confirm)
    }
}

pub(in crate::level) fn confirm_input_scene() -> impl Scene {
    let input_map = InputMap::new([(UiAction::Confirm, KeyCode::KeyE)]);
    bsn! {
        input_map
        ActionState::<UiAction>::default()
        ConfirmArmed::default()
    }
}

