use super::{
    GamePhase, RoadId,
    navigation::{self, DestinationChoices},
};
use bevy::{
    prelude::*,
    ui_widgets::{Activate, Button},
};

#[derive(Component)]
pub(super) struct TravelUiRoot;

#[derive(Component, Message, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TravelButtonAction {
    Destination(RoadId),
    Close,
}

pub(super) fn spawn_ui(mut commands: Commands, choices: Res<DestinationChoices>) {
    commands
        .spawn((
            TravelUiRoot,
            DespawnOnExit(GamePhase::ChoosingDestination),
            Name::new("Travel UI"),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            GlobalZIndex(100),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
            Pickable::default(),
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        min_width: px(320),
                        max_width: percent(90),
                        padding: UiRect::all(px(24)),
                        row_gap: px(16),
                        border_radius: BorderRadius::all(px(12)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.14, 0.2)),
                ))
                .with_children(|panel| {
                    panel.spawn(text("Choose a destination", 28.0));
                    for &destination in choices.0 {
                        spawn_button(
                            panel,
                            destination.0,
                            TravelButtonAction::Destination(destination),
                        );
                    }
                    spawn_button(panel, "Cancel (Esc)", TravelButtonAction::Close);
                });
        });
}

pub(super) fn close_on_escape(keys: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if keys.just_pressed(KeyCode::Escape) {
        commands.queue(navigation::close_ui);
    }
}

fn text(label: &str, font_size: f32) -> impl Bundle {
    (
        Text::new(label),
        TextFont {
            font_size: FontSize::Px(font_size),
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.96, 1.0)),
        Pickable::IGNORE,
    )
}

fn spawn_button(parent: &mut ChildSpawnerCommands, label: &str, action: TravelButtonAction) {
    parent
        .spawn((
            Button,
            action,
            Node {
                min_height: px(46),
                padding: UiRect::axes(px(18), px(10)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.35, 0.5)),
        ))
        .observe(activate_button)
        .with_child(text(label, 20.0));
}

fn activate_button(
    activate: On<Activate>,
    actions: Query<&TravelButtonAction>,
    mut requests: MessageWriter<TravelButtonAction>,
) {
    if let Ok(&action) = actions.get(activate.entity) {
        // Process during Update, after StateTransition, so E stays blocked on the closing frame.
        requests.write(action);
    }
}

pub(super) fn handle_buttons(
    mut actions: MessageReader<TravelButtonAction>,
    mut commands: Commands,
) {
    for &action in actions.read() {
        commands.queue(move |world: &mut World| match action {
            TravelButtonAction::Destination(destination) => {
                navigation::select_destination(world, destination);
            }
            TravelButtonAction::Close => navigation::close_ui(world),
        });
    }
}
