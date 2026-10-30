#[cfg(test)]
mod tests;

use super::SceneRoot;
use crate::config::GameState;
use bevy::{
    prelude::*,
    ui_widgets::{Activate, Button},
};

#[derive(Component, Clone, Default)]
pub(super) struct MainMenuRoot;

pub(super) fn setup_menu(world: &mut World) {
    world.spawn_scene(menu_scene()).unwrap();
}

pub(super) fn cleanup_menu(
    mut commands: Commands,
    roots: Query<Entity, (With<SceneRoot>, With<MainMenuRoot>)>,
) {
    for root in &roots {
        commands.entity(root).despawn();
    }
}

fn menu_scene() -> impl Scene {
    bsn! {
        #MainMenu
        SceneRoot
        MainMenuRoot
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(32),
        }
        BackgroundColor(Color::srgb(0.04, 0.05, 0.08))
        Children [
            Text::new("Intern Judge")
            TextFont {
                font_size: FontSize::Px(56.0),
            }
            TextColor(Color::srgb(0.95, 0.96, 1.0))
            Pickable::IGNORE
            --
            Button
            Node {
                padding: UiRect::axes(px(32), px(16)),
                border_radius: BorderRadius::all(px(8)),
            }
            BackgroundColor(Color::srgb(0.18, 0.38, 0.7))
            on(start_game)
            Children [
                Text::new("Start Game")
                TextFont {
                    font_size: FontSize::Px(28.0),
                }
                TextColor(Color::srgb(0.95, 0.96, 1.0))
                Pickable::IGNORE
            ]
        ]
    }
}

fn start_game(
    _: On<Activate>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if *state.get() == GameState::MainMenu {
        next_state.set(GameState::Playing);
    }
}
