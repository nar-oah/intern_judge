use super::{DialogueBox, DialogueOptions, DialoguePrompt};
use crate::level::ui::confirm_input_scene;
use bevy::{
    prelude::*,
    text::FontSourceTemplate,
    ui_widgets::{Activate, Button},
};
use bevy_replique::prelude::{ResumeDialogue, ResumeInput};

#[derive(Component, Clone, Copy, Default)]
pub(super) struct DialogueChoiceButton(pub(super) usize);

pub(in crate::level) fn dialogue_box_scene() -> impl Scene {
    bsn! {
        #DialogueBox
        DialogueBox::default()
        @confirm_input_scene()
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
            @dialogue_text_scene("", 28.0)
            --
            DialogueOptions
            Node {
                flex_wrap: FlexWrap::Wrap,
                column_gap: px(16),
                row_gap: px(8),
            }
            Pickable::IGNORE
        ]
    }
}

pub(super) fn dialogue_choice_scene(label: &str, index: usize) -> impl Scene + use<> {
    bsn! {
        Button
        DialogueChoiceButton(index)
        Node {
            padding: UiRect::axes(px(20), px(12)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(px(6)),
        }
        BackgroundColor(Color::srgb(0.2, 0.28, 0.42))
        on(select_choice)
        Children [ @dialogue_text_scene(label, 22.0) ]
    }
}

fn dialogue_text_scene(label: &str, size: f32) -> impl Scene + use<> {
    let text = Text::new(label);
    bsn! {
        text
        TextFont {
            font: FontSourceTemplate::Handle("fonts/court-dialogue.ttf"),
            font_size: FontSize::Px(size),
        }
        TextColor(Color::srgb(0.95, 0.96, 1.0))
        Pickable::IGNORE
    }
}

fn select_choice(
    activate: On<Activate>,
    buttons: Query<&DialogueChoiceButton>,
    mut dialogs: Query<&mut DialogueBox>,
    mut resume: MessageWriter<ResumeDialogue>,
) {
    let Ok(choice) = buttons.get(activate.entity) else {
        return;
    };
    let Ok(mut dialog) = dialogs.single_mut() else {
        return;
    };
    if let Some(token) = dialog.choices.take() {
        resume.write(ResumeDialogue {
            token,
            input: ResumeInput::Select(choice.0),
        });
    }
}
