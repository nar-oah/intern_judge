#[cfg(test)]
mod tests;
pub(super) mod ui;

use bevy::prelude::*;
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
