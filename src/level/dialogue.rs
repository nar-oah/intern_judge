#[cfg(test)]
mod tests;
pub(super) mod ui;
use super::{InteractRequested, RoadEffect, RoadEffects, RoadId};
use crate::player::{Player, PlayerAction};
use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;
use bevy_replique::prelude::*;
use leafwing_input_manager::prelude::ActionState;

#[derive(Component, Clone, Copy)]
pub(super) struct CourtDialogue(pub(super) &'static str);

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

pub(super) fn start_court_dialogue(
    request: On<InteractRequested>,
    targets: Query<&CourtDialogue>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut players: Query<&mut LinearVelocity, (With<Player>, Without<DialogueRunner>)>,
    mut start: MessageWriter<StartDialogue>,
) {
    let Ok(mut velocity) = players.get_mut(request.player) else {
        return;
    };
    let node = targets.get(request.entity).unwrap().0;
    velocity.x = 0.0;
    commands
        .entity(request.player)
        .insert(DialogueRunner::new(assets.load("dialogue/court.rep")));
    commands.spawn_scene(ui::dialogue_box_scene());
    start.write(StartDialogue {
        runner: request.player,
        node: node.into(),
    });
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
    players: Query<&ActionState<PlayerAction>, (With<Player>, With<DialogueRunner>)>,
    mut dialogs: Query<&mut DialogueBox>,
    mut resume: MessageWriter<ResumeDialogue>,
) {
    let Ok(actions) = players.single() else {
        return;
    };
    if !actions.just_pressed(&PlayerAction::NextDialogue) {
        return;
    }
    let Ok(mut dialog) = dialogs.single_mut() else {
        return;
    };
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
fn verdict(In(guilty): In<bool>, mut effects: ResMut<RoadEffects>) {
    let (road, effect) = if guilty {
        (RoadId::ExampleRoad, RoadEffect::FallingObject)
    } else {
        (RoadId::MarketRoad, RoadEffect::ConstructionDetour)
    };
    let effects = effects.0.entry(road).or_default();
    if !effects.contains(&effect) {
        effects.push(effect);
    }
}
