#[cfg(test)]
mod tests;

use super::{RoadId, SceneRoot, map::MapLayout, navigation};
use crate::config::RoadGraph;
use bevy::{
    prelude::*,
    ui_widgets::{Activate, Button},
};

#[derive(Component, Clone, Default)]
pub(super) struct TravelUiRoot(pub(super) Vec<RoadId>);

#[derive(Component, Clone, Copy, Default, Debug, PartialEq, Eq)]
pub(super) struct TravelButtonAction(pub(super) RoadId);

#[derive(Component, Clone, Copy)]
pub(super) struct MapRoadNode(pub(super) RoadId);

#[derive(Component, Clone, Copy)]
pub(super) struct MapConnection {
    pub(super) from: RoadId,
    pub(super) to: RoadId,
}

// All map geometry uses the same viewport unit, preserving angles on resize.
const CANVAS_SIZE: f32 = 72.0;
const MAP_RADIUS: f32 = 24.0;
const NODE_SIZE: Vec2 = Vec2::new(18.0, 7.0);

pub(super) fn travel_ui_scene(
    current: RoadId,
    destinations: Vec<RoadId>,
    graph: &RoadGraph,
    layout: &MapLayout,
) -> impl Scene + use<> {
    let radius = layout
        .positions
        .values()
        .map(|position| position.length())
        .fold(1.0, f32::max);
    let project = |road: RoadId| {
        Vec2::splat(CANVAS_SIZE / 2.0) + layout.positions[&road] * (MAP_RADIUS / radius)
    };
    let mut edges = graph
        .roads
        .iter()
        .flat_map(|(&from, roads)| roads.iter().map(move |&to| (from, to)))
        .filter(|(from, to)| {
            from != to && layout.positions.contains_key(from) && layout.positions.contains_key(to)
        })
        .collect::<Vec<_>>();
    edges.sort_unstable();
    edges.dedup();
    let connections = edges
        .into_iter()
        .map(|(from, to)| {
            connection_scene(
                MapConnection { from, to },
                project(from),
                project(to),
                current,
                &destinations,
            )
        })
        .collect::<Vec<_>>();
    let mut roads = layout.positions.keys().copied().collect::<Vec<_>>();
    roads.sort_unstable();
    let nodes = roads
        .into_iter()
        .map(|road| {
            road_node_scene(
                MapRoadNode(road),
                project(road),
                current,
                destinations.contains(&road),
            )
        })
        .collect::<Vec<_>>();

    bsn! {
        #TravelUi
        SceneRoot
        TravelUiRoot(destinations)
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
        }
        GlobalZIndex(100)
        BackgroundColor(Color::srgb(0.04, 0.05, 0.08))
        Pickable::default()
        Children [
            Node {
                position_type: PositionType::Absolute,
                left: vmin(3),
                top: vmin(3),
                flex_direction: FlexDirection::Column,
                row_gap: vmin(1),
            }
            Pickable::IGNORE
            Children [
                @text_scene("Road map", 3.5)
                --
                @text_scene("Gold: Court | Blue: Current", 1.8)
                --
                @text_scene("Green border: Available | Arrows: Travel direction", 1.8)
            ]
            --
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                top: percent(58),
                width: vmin(CANVAS_SIZE),
                height: vmin(CANVAS_SIZE),
            }
            UiTransform::from_translation(Val2::percent(-50.0, -50.0))
            Pickable::IGNORE
            Children [
                { connections }
                -- { nodes }
            ]
        ]
    }
}

fn text_scene(label: &str, font_size: f32) -> impl Scene + use<> {
    let text = Text::new(label);
    bsn! {
        text
        TextFont {
            font_size: FontSize::VMin(font_size),
        }
        TextColor(Color::srgb(0.95, 0.96, 1.0))
        Pickable::IGNORE
    }
}

fn road_node_scene(
    node: MapRoadNode,
    position: Vec2,
    current: RoadId,
    available: bool,
) -> impl Scene {
    let label = match node.0 {
        RoadId::ExampleRoad => "Example Road",
        RoadId::MarketRoad => "Market Road",
        RoadId::Court => "Court",
    };
    let color = if node.0 == current {
        Color::srgb(0.18, 0.38, 0.7)
    } else if node.0 == RoadId::Court {
        Color::srgb(0.65, 0.45, 0.12)
    } else if available {
        Color::srgb(0.12, 0.4, 0.28)
    } else {
        Color::srgb(0.19, 0.21, 0.27)
    };
    let border = if available {
        Color::srgb(0.35, 0.95, 0.55)
    } else {
        Color::srgb(0.4, 0.44, 0.52)
    };
    let destination = node.0;
    let action = available.then(|| {
        bsn! {
            Button
            TravelButtonAction(destination)
            on(activate_button)
        }
    });
    let picking = if available {
        Pickable::default()
    } else {
        Pickable::IGNORE
    };
    bsn! {
        node
        Node {
            position_type: PositionType::Absolute,
            left: vmin(position.x),
            top: vmin(position.y),
            width: vmin(NODE_SIZE.x),
            height: vmin(NODE_SIZE.y),
            border: UiRect::all(vmin(0.3)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(vmin(1.5)),
        }
        UiTransform::from_translation(Val2::percent(-50.0, -50.0))
        ZIndex(1)
        BackgroundColor(color)
        BorderColor::all(border)
        picking
        @{action}
        Children [ @text_scene(label, 1.8) ]
    }
}

fn connection_scene(
    connection: MapConnection,
    from: Vec2,
    to: Vec2,
    current: RoadId,
    destinations: &[RoadId],
) -> impl Scene + use<> {
    let direction = (to - from).normalize_or_zero();
    // Stop at the rectangular node boundary so the arrow remains visible.
    let clearance = |direction: Vec2| (NODE_SIZE / 2.0 / direction.abs()).min_element() + 0.6;
    let start = from + direction * clearance(direction);
    let tip = to - direction * clearance(direction);
    let arrow_back = tip - direction * 1.4;
    let side = direction.perp() * 0.8;
    let color = if connection.from == current && destinations.contains(&connection.to) {
        Color::srgb(0.35, 0.75, 0.5)
    } else {
        Color::srgb(0.35, 0.4, 0.5)
    };
    bsn! {
        connection
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
        }
        Pickable::IGNORE
        Children [
            @line_scene(start, tip, color)
            --
            @line_scene(arrow_back + side, tip, color)
            --
            @line_scene(arrow_back - side, tip, color)
        ]
    }
}

fn line_scene(from: Vec2, to: Vec2, color: Color) -> impl Scene {
    let midpoint = (from + to) / 2.0;
    let delta = to - from;
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: vmin(midpoint.x),
            top: vmin(midpoint.y),
            width: vmin(delta.length()),
            height: vmin(0.25),
            min_height: px(1),
        }
        UiTransform {
            translation: Val2::percent(-50.0, -50.0),
            rotation: Rot2::radians(delta.y.atan2(delta.x)),
        }
        BackgroundColor(color)
        Pickable::IGNORE
    }
}

fn activate_button(activate: On<Activate>, mut commands: Commands) {
    let button = activate.entity;
    commands.queue(move |world: &mut World| navigation::select_destination(world, button));
}
