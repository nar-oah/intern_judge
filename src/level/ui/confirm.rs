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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{advance, finish_app, scene_app};
    use bevy::input::InputPlugin;

    #[derive(Resource, Default)]
    struct Confirmations(usize);

    #[test]
    fn new_ui_waits_for_release_before_accepting_a_confirm_press() {
        let mut app = scene_app();
        app.add_plugins((InputPlugin, InputManagerPlugin::<UiAction>::default()))
            .init_resource::<Confirmations>()
            .add_systems(
                Update,
                |mut inputs: Query<(&ActionState<UiAction>, &mut ConfirmArmed)>,
                 mut confirmations: ResMut<Confirmations>| {
                    for (actions, mut confirm) in &mut inputs {
                        if confirm.just_pressed(actions) {
                            confirmations.0 += 1;
                        }
                    }
                },
            );
        finish_app(&mut app);
        app.world_mut().spawn_scene(confirm_input_scene()).unwrap();

        KeyCode::KeyE.press(app.world_mut());
        advance(&mut app, 4);
        assert_eq!(app.world().resource::<Confirmations>().0, 0);

        KeyCode::KeyE.release(app.world_mut());
        app.update();
        assert_eq!(app.world().resource::<Confirmations>().0, 0);
        KeyCode::KeyE.press(app.world_mut());
        app.update();
        assert_eq!(app.world().resource::<Confirmations>().0, 1);
        advance(&mut app, 4);
        assert_eq!(app.world().resource::<Confirmations>().0, 1);
    }
}
