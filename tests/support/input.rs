use bevy::prelude::*;
use leafwing_input_manager::prelude::Buttonlike;

/// Release first so a held key produces a fresh input edge.
pub(crate) fn press_key(app: &mut App, key: KeyCode) {
    key.release(app.world_mut());
    app.update();
    key.press(app.world_mut());
    app.update();
}

pub(crate) fn press_e(app: &mut App) {
    press_key(app, KeyCode::KeyE);
}
