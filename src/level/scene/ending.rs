use super::super::SceneRoot;
use bevy::{prelude::*, text::FontSourceTemplate};

#[derive(Component, Clone, Default)]
pub(crate) struct InternshipEnded;

pub(in crate::level) fn internship_ended_scene() -> impl Scene {
    bsn! {
        #InternshipEnded
        SceneRoot
        InternshipEnded
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        }
        BackgroundColor(Color::srgb(0.04, 0.05, 0.08))
        Children [
            Text::new("实习结束")
            TextFont {
                font: FontSourceTemplate::Handle("fonts/court-dialogue.ttf"),
                font_size: FontSize::Px(48.0),
            }
            TextColor(Color::srgb(0.95, 0.96, 1.0))
            Pickable::IGNORE
        ]
    }
}
