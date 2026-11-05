use crate::level::RoadId;
use bevy::{prelude::*, text::FontSourceTemplate};

pub(in crate::level) fn case_hud_scene(road: RoadId) -> impl Scene {
    bsn! {
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
            Text::new(format!("今日涉案路段：{road:?}"))
            TextFont {
                font: FontSourceTemplate::Handle("fonts/court-dialogue.ttf"),
                font_size: FontSize::Px(22.0),
            }
            TextColor(Color::srgb(0.95, 0.96, 1.0))
            Pickable::IGNORE
        ]
    }
}
