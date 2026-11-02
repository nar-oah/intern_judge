use bevy::{
    prelude::*,
    ui_widgets::{Activate, Button},
};
use bevy_replique::prelude::*;

#[derive(EntityEvent)]
pub(super) struct DialogueResponse {
    pub entity: Entity,
    pub token: DialogueToken,
    pub input: ResumeInput,
}

#[derive(Component, Clone)]
pub(super) struct DialogueButton {
    pub token: DialogueToken,
    pub choice: Option<usize>,
}

pub(super) fn dialogue_box_scene(
    font: Handle<Font>,
    speaker: &str,
    text: &str,
    token: DialogueToken,
    choices: Option<&[DialogueChoice]>,
) -> impl Scene + use<> {
    let buttons = match choices {
        Some(choices) => choices
            .iter()
            .filter(|choice| choice.available)
            .map(|choice| button_scene(font.clone(), &choice.text, token, Some(choice.index)))
            .collect::<Vec<_>>(),
        None => vec![button_scene(font.clone(), "按 E 继续", token, None)],
    };

    bsn! {
        #DialogueBox
        Node {
            position_type: PositionType::Absolute,
            left: percent(8),
            right: percent(8),
            bottom: px(24),
            padding: UiRect::all(px(20)),
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            border_radius: BorderRadius::all(px(8)),
        }
        GlobalZIndex(300)
        BackgroundColor(Color::srgb(0.08, 0.1, 0.16))
        Children [
            @text_scene(font.clone(), speaker, 24.0)
            --
            @text_scene(font, text, 22.0)
            --
            Node {
                flex_wrap: FlexWrap::Wrap,
                column_gap: px(16),
                row_gap: px(12),
            }
            Children [ { buttons } ]
        ]
    }
}

fn text_scene(font: Handle<Font>, text: &str, size: f32) -> impl Scene + use<> {
    let text = Text::new(text);
    let font = bevy::text::FontSourceTemplate::Handle(font.into());
    bsn! {
        text
        TextFont {
            font,
            font_size: FontSize::Px(size),
        }
        TextColor(Color::srgb(0.95, 0.96, 1.0))
        Pickable::IGNORE
    }
}

fn button_scene(
    font: Handle<Font>,
    label: &str,
    token: DialogueToken,
    choice: Option<usize>,
) -> impl Scene + use<> {
    let action = DialogueButton { token, choice };
    bsn! {
        Button
        action
        Node {
            padding: UiRect::axes(px(20), px(10)),
            border_radius: BorderRadius::all(px(6)),
        }
        BackgroundColor(Color::srgb(0.18, 0.38, 0.7))
        on(respond)
        Children [ @text_scene(font, label, 22.0) ]
    }
}

fn respond(activate: On<Activate>, buttons: Query<&DialogueButton>, mut commands: Commands) {
    let Ok(button) = buttons.get(activate.entity) else {
        return;
    };
    commands.trigger(DialogueResponse {
        entity: button.token.runner(),
        token: button.token,
        input: button
            .choice
            .map_or(ResumeInput::Advance, ResumeInput::Select),
    });
}
