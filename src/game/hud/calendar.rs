use bevy::prelude::*;

#[derive(Component, Clone, Default)]
pub(in crate::game) struct CalendarHud;

#[derive(Component, Clone, Default)]
pub(in crate::game) struct CalendarHudText;

pub(super) fn calendar_hud_scene() -> impl Scene {
    bsn! {
        #CalendarHudRoot
        CalendarHud
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            left: px(16),
            padding: UiRect::all(px(12)),
            border_radius: BorderRadius::all(px(6)),
        }
        GlobalZIndex(200)
        BackgroundColor(Color::srgb(0.12, 0.14, 0.2))
        Pickable::IGNORE
        Children [
            CalendarHudText
            Text::default()
            TextFont { font_size: FontSize::Px(22.0) }
            TextColor(Color::srgb(0.95, 0.96, 1.0))
            Pickable::IGNORE
        ]
    }
}
