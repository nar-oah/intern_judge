use super::{RoadId, SceneRoot, navigation};
use bevy::{
    prelude::*,
    ui_widgets::{Activate, Button},
};

#[derive(Component, Clone, Default)]
pub(super) struct TravelUiRoot(pub(super) Vec<RoadId>);

#[derive(Component, Clone, Copy, Default, Debug, PartialEq, Eq)]
pub(super) struct TravelButtonAction(pub(super) RoadId);

pub(super) fn travel_ui_scene(destinations: Vec<RoadId>) -> impl Scene {
    let buttons = destinations
        .iter()
        .copied()
        .map(destination_button_scene)
        .collect::<Vec<_>>();

    bsn! {
        #TravelUi
        SceneRoot
        TravelUiRoot(destinations)
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        }
        GlobalZIndex(100)
        BackgroundColor(Color::srgb(0.04, 0.05, 0.08))
        Pickable::default()
        Children [
            Node {
                flex_direction: FlexDirection::Column,
                min_width: px(320),
                max_width: percent(90),
                padding: UiRect::all(px(24)),
                row_gap: px(16),
                border_radius: BorderRadius::all(px(12)),
            }
            BackgroundColor(Color::srgb(0.12, 0.14, 0.2))
            Children [
                @text_scene("Choose a destination", 28.0)
                --
                { buttons }
            ]
        ]
    }
}

fn text_scene(label: &str, font_size: f32) -> impl Scene {
    let text = Text::new(label);
    bsn! {
        text
        TextFont {
            font_size: FontSize::Px(font_size),
        }
        TextColor(Color::srgb(0.95, 0.96, 1.0))
        Pickable::IGNORE
    }
}

fn destination_button_scene(destination: RoadId) -> impl Scene {
    let label = match destination {
        RoadId::ExampleRoad => "example_road",
        RoadId::MarketRoad => "market_road",
        RoadId::Court => "court",
    };
    bsn! {
        Button
        TravelButtonAction(destination)
        Node {
            min_height: px(46),
            padding: UiRect::axes(px(18), px(10)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(px(6)),
        }
        BackgroundColor(Color::srgb(0.2, 0.35, 0.5))
        on(activate_button)
        Children [ @text_scene(label, 20.0) ]
    }
}

fn activate_button(activate: On<Activate>, mut commands: Commands) {
    let button = activate.entity;
    commands.queue(move |world: &mut World| navigation::select_destination(world, button));
}
