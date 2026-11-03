#[cfg(test)]
mod tests;

use super::InteractRequested;
use crate::player::Player;
use avian2d::prelude::LinearVelocity;
use bevy::{prelude::*, text::FontSourceTemplate};
use bevy_replique::prelude::*;

const CHOICE_KEYS: [(KeyCode, &str); 3] = [
    (KeyCode::ArrowLeft, "← 左方向键"),
    (KeyCode::ArrowRight, "→ 右方向键"),
    (KeyCode::KeyE, "E 键"),
];

#[derive(Component, Clone)]
struct DialogueBox {
    runner: Entity,
    choices: Option<DialogueChoices>,
}

#[derive(Component, Clone, Default)]
struct DialoguePrompt;

#[derive(Component, Clone, Default)]
struct DialogueOptions;

pub(super) fn setup(app: &mut App) {
    app.add_plugins(RepliquePlugin)
        .add_dialogue_command(verdict)
        .add_observer(cleanup_dialogue)
        .add_systems(
            Update,
            choose_dialogue.run_if(any_with_component::<DialogueBox>),
        )
        .add_systems(
            PostUpdate,
            (show_dialogue, close_dialogue)
                .chain()
                .after(DialogueSystem::Commands),
        );
}

pub(super) fn open_dialogue(
    request: On<InteractRequested>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut players: Query<&mut LinearVelocity, (With<Player>, Without<DialogueRunner>)>,
    mut start: MessageWriter<StartDialogue>,
) {
    let Ok(mut velocity) = players.get_mut(request.player) else {
        return;
    };
    velocity.x = 0.0;
    commands
        .entity(request.player)
        .insert(DialogueRunner::new(assets.load("dialogue/court.rep")));
    commands.spawn_scene(dialogue_box_scene(request.player));
    start.write(StartDialogue {
        runner: request.player,
        node: "verdict".into(),
    });
}

fn dialogue_box_scene(runner: Entity) -> impl Scene {
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

fn show_dialogue(
    mut lines: MessageReader<DialogueLine>,
    mut choices: MessageReader<DialogueChoices>,
    mut dialogs: Query<&mut DialogueBox>,
    mut prompts: Query<&mut Text, With<DialoguePrompt>>,
    mut options: Query<&mut Text, (With<DialogueOptions>, Without<DialoguePrompt>)>,
    mut resume: MessageWriter<ResumeDialogue>,
) {
    let Ok(mut dialog) = dialogs.single_mut() else {
        lines.clear();
        choices.clear();
        return;
    };
    for line in lines.read() {
        if line.token.runner() == dialog.runner {
            if let Ok(mut text) = prompts.single_mut() {
                text.0.clone_from(&line.text);
            }
            // Keep the prompt visible and advance straight to the script's choices.
            resume.write(ResumeDialogue {
                token: line.token,
                input: ResumeInput::Advance,
            });
        }
    }
    for choices in choices.read() {
        if choices.token.runner() == dialog.runner {
            if let Ok(mut text) = options.single_mut() {
                text.0 = choices
                    .choices
                    .iter()
                    .zip(CHOICE_KEYS)
                    .filter(|(choice, _)| choice.available)
                    .map(|(choice, (_, key))| format!("{key}：{}", choice.text))
                    .collect::<Vec<_>>()
                    .join("    ");
            }
            dialog.choices = Some(choices.clone());
        }
    }
}

fn choose_dialogue(
    keys: Res<ButtonInput<KeyCode>>,
    mut dialogs: Query<&mut DialogueBox>,
    mut resume: MessageWriter<ResumeDialogue>,
) {
    for mut dialog in &mut dialogs {
        let Some(choices) = &dialog.choices else {
            continue;
        };
        let selection = choices
            .choices
            .iter()
            .zip(CHOICE_KEYS)
            .find(|(choice, (key, _))| choice.available && keys.just_pressed(*key));
        if let Some((choice, _)) = selection {
            resume.write(ResumeDialogue {
                token: choices.token,
                input: ResumeInput::Select(choice.index),
            });
            dialog.choices = None;
        }
    }
}

fn close_dialogue(
    mut commands: Commands,
    mut finished: MessageReader<DialogueFinished>,
    dialogs: Query<&DialogueBox>,
) {
    for finished in finished.read() {
        if dialogs
            .iter()
            .any(|dialog| dialog.runner == finished.runner)
        {
            commands.entity(finished.runner).remove::<DialogueRunner>();
        }
    }
}

fn cleanup_dialogue(
    removed: On<Remove<DialogueRunner>>,
    mut commands: Commands,
    dialogs: Query<(Entity, &DialogueBox)>,
) {
    for (entity, dialog) in &dialogs {
        if dialog.runner == removed.entity {
            commands.entity(entity).despawn();
        }
    }
}

#[replique_command]
fn verdict(In(result): In<String>) {
    info!("判决结果：{result}");
}
