#[cfg(test)]
mod tests;
pub(super) mod ui;
use super::{
    RoadEffect, RoadEffects, RoadId,
    day::DayEndRequested,
    ui::{ConfirmArmed, UiAction},
};
use bevy::prelude::*;
use bevy_replique::prelude::*;
use crate::game::calendar::GameCalendar;
use leafwing_input_manager::prelude::ActionState;

#[derive(Component, Clone, Default)]
struct DialogueBox {
    line: Option<DialogueToken>,
    choices: Option<DialogueToken>,
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
            advance_dialogue.run_if(any_with_component::<DialogueBox>),
        )
        .add_systems(
            PostUpdate,
            (show_dialogue, close_dialogue)
                .chain()
                .after(DialogueSystem::Commands),
        );
}

fn show_dialogue(
    mut commands: Commands,
    mut lines: MessageReader<DialogueLine>,
    mut choices: MessageReader<DialogueChoices>,
    mut dialogs: Query<&mut DialogueBox>,
    mut prompts: Query<&mut Text, With<DialoguePrompt>>,
    options: Query<Entity, With<DialogueOptions>>,
) {
    let Ok(mut dialog) = dialogs.single_mut() else {
        lines.clear();
        choices.clear();
        return;
    };
    for line in lines.read() {
        if let Ok(mut text) = prompts.single_mut() {
            text.0.clone_from(&line.text);
        }
        if let Ok(options) = options.single() {
            commands.entity(options).despawn_children();
        }
        dialog.line = Some(line.token);
        dialog.choices = None;
    }
    for choices in choices.read() {
        if let Ok(options) = options.single() {
            commands.entity(options).despawn_children();
            for choice in choices.choices.iter().filter(|choice| choice.available) {
                commands
                    .spawn_scene(ui::dialogue_choice_scene(&choice.text, choice.index))
                    .insert(ChildOf(options));
            }
        }
        dialog.line = None;
        dialog.choices = Some(choices.token);
    }
}

fn advance_dialogue(
    mut dialogs: Query<(&ActionState<UiAction>, &mut ConfirmArmed, &mut DialogueBox)>,
    mut resume: MessageWriter<ResumeDialogue>,
) {
    let Ok((actions, mut confirm, mut dialog)) = dialogs.single_mut() else {
        return;
    };
    if !confirm.just_pressed(actions) {
        return;
    }
    if let Some(token) = dialog.line.take() {
        resume.write(ResumeDialogue {
            token,
            input: ResumeInput::Advance,
        });
    }
}

fn close_dialogue(mut commands: Commands, mut finished: MessageReader<DialogueFinished>) {
    for finished in finished.read() {
        commands.entity(finished.runner).remove::<DialogueRunner>();
    }
}

fn cleanup_dialogue(
    _: On<Remove<DialogueRunner>>,
    mut commands: Commands,
    dialogs: Query<Entity, With<DialogueBox>>,
) {
    if let Ok(entity) = dialogs.single() {
        commands.entity(entity).despawn();
    }
}

#[replique_command]
fn verdict(
    In(guilty): In<bool>,
    calendar: Res<GameCalendar>,
    mut effects: ResMut<RoadEffects>,
    mut commands: Commands,
) {
    if calendar.is_weekend() {
        return;
    }
    let (road, effect) = if guilty {
        (RoadId::ExampleRoad, RoadEffect::FallingObject)
    } else {
        (RoadId::MarketRoad, RoadEffect::ConstructionDetour)
    };
    let effects = effects.0.entry(road).or_default();
    if !effects.contains(&effect) {
        effects.push(effect);
    }
    commands.insert_resource(DayEndRequested);
}
