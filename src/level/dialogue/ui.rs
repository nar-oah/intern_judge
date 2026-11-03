use super::{DialogueBox, DialogueOptions, DialoguePrompt};
use bevy::{prelude::*, text::FontSourceTemplate};

pub(in crate::level) fn dialogue_box_scene(runner: Entity) -> impl Scene {
    let dialog = DialogueBox {
        runner,
        choices: None,
    };
    bsn! {
        #DialogueBox
        dialog
        Node {
            position_type: PositionType::Absolute,
            left: percent(10),
            right: percent(10),
            bottom: px(24),
            padding: UiRect::all(px(24)),
            flex_direction: FlexDirection::Column,
            row_gap: px(16),
            border_radius: BorderRadius::all(px(8)),
        }
        GlobalZIndex(300)
        BackgroundColor(Color::srgb(0.08, 0.1, 0.16))
        Pickable::IGNORE
        Children [
            DialoguePrompt
            @dialogue_text_scene(28.0)
            --
            DialogueOptions
            @dialogue_text_scene(22.0)
        ]
    }
}

fn dialogue_text_scene(size: f32) -> impl Scene {
    bsn! {
        Text::default()
        TextFont {
            font: FontSourceTemplate::Handle("fonts/court-dialogue.ttf"),
            font_size: FontSize::Px(size),
        }
        TextColor(Color::srgb(0.95, 0.96, 1.0))
        Pickable::IGNORE
    }
}
