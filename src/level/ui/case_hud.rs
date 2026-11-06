use crate::{
    config::GameState,
    level::{CurrentCase, RoadId},
};
use bevy::{prelude::*, text::FontSourceTemplate};

#[derive(Component, Clone, Default)]
struct CaseHud;

#[derive(Component, Clone, Default)]
struct CaseHudText;

pub(super) fn setup(app: &mut App) {
    app.add_systems(OnEnter(GameState::Playing), setup_case_hud)
        .add_systems(OnExit(GameState::Playing), cleanup_case_hud)
        .add_systems(
            Update,
            update_case_hud
                .run_if(resource_exists_and_changed::<CurrentCase>)
                .run_if(in_state(GameState::Playing)),
        );
}

fn setup_case_hud(mut commands: Commands, current_case: Option<Res<CurrentCase>>) {
    let road = current_case
        .as_deref()
        .and_then(|current| current.0.as_ref())
        .map(|selected| selected.road);
    commands.spawn_scene(case_hud_scene(road));
}

fn cleanup_case_hud(mut commands: Commands, roots: Query<Entity, With<CaseHud>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
}

fn update_case_hud(
    current_case: Res<CurrentCase>,
    mut roots: Query<&mut Visibility, With<CaseHud>>,
    mut labels: Query<&mut Text, With<CaseHudText>>,
) {
    let road = current_case.0.as_ref().map(|selected| selected.road);
    for mut visibility in &mut roots {
        *visibility = if road.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for mut text in &mut labels {
        text.0 = road.map_or_else(String::new, |road| format!("今日涉案路段：{road:?}"));
    }
}

fn case_hud_scene(road: Option<RoadId>) -> impl Scene {
    let visibility = if road.is_some() {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    bsn! {
        #CaseHudRoot
        CaseHud
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
        }
        visibility
        Pickable::IGNORE
        Children [
            #CaseHud
            Node {
                position_type: PositionType::Absolute,
                top: px(16),
                left: percent(50),
                padding: UiRect::all(px(12)),
                border_radius: BorderRadius::all(px(6)),
            }
            UiTransform::from_translation(Val2::percent(-50.0, 0.0))
            GlobalZIndex(200)
            BackgroundColor(Color::srgb(0.12, 0.14, 0.2))
            Pickable::IGNORE
            Children [
                CaseHudText
                Text::new(road.map_or_else(String::new, |road| format!("今日涉案路段：{road:?}")))
                TextFont {
                    font: FontSourceTemplate::Handle("fonts/court-dialogue.ttf"),
                    font_size: FontSize::Px(22.0),
                }
                TextColor(Color::srgb(0.95, 0.96, 1.0))
                Pickable::IGNORE
            ]
        ]
    }
}
