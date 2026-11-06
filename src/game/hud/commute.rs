use bevy::prelude::*;

#[derive(Component, Clone, Default)]
pub(in crate::game) struct CommuteHud;

#[derive(Component, Clone, Default)]
pub(in crate::game) struct CommuteHudText;

#[derive(Component, Clone, Default)]
pub(in crate::game) struct StaminaBar;

pub(super) fn commute_hud_scene() -> impl Scene {
    bsn! {
        #CommuteHudRoot
        CommuteHud
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            right: px(16),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            padding: UiRect::all(px(12)),
            border_radius: BorderRadius::all(px(6)),
        }
        GlobalZIndex(200)
        BackgroundColor(Color::srgb(0.12, 0.14, 0.2))
        Pickable::IGNORE
        Children [
            CommuteHudText
            Text::default()
            TextFont { font_size: FontSize::Px(22.0) }
            TextColor(Color::srgb(0.95, 0.96, 1.0))
            Pickable::IGNORE
            --
            Node {
                width: px(200),
                height: px(12),
                border_radius: BorderRadius::all(px(6)),
                overflow: Overflow::clip(),
            }
            BackgroundColor(Color::srgb(0.04, 0.05, 0.08))
            Pickable::IGNORE
            Children [
                StaminaBar
                Node { width: percent(100), height: percent(100) }
                BackgroundColor(Color::srgb(0.3, 0.8, 0.45))
                Pickable::IGNORE
            ]
        ]
    }
}
