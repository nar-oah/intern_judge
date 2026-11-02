#[cfg(test)]
mod tests;

use super::{
    InteractRequested,
    dialogue_ui::{DialogueResponse, dialogue_box_scene},
};
use crate::player::{DialogueLocked, Player, PlayerAction, Stunned};
use avian2d::prelude::{CollidingEntities, LinearVelocity};
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_replique::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CourtNpc {
    Prosecution,
    Defense,
}

impl CourtNpc {
    fn node(self) -> &'static str {
        match self {
            Self::Prosecution => "prosecution",
            Self::Defense => "defense",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Verdict {
    Guilty,
    NotGuilty,
}

#[derive(Clone, Copy)]
enum Waiting {
    Line(DialogueToken),
    Choices(DialogueToken),
}

#[derive(Component, Clone, Default)]
pub(super) struct CourtHearing {
    heard: [bool; 2],
    verdict: Option<Verdict>,
    player: Option<Entity>,
    panel: Option<Entity>,
    waiting: Option<Waiting>,
}

// Panels must be UI roots, so use a separate relationship for scene cleanup.
#[derive(Component, Clone)]
#[relationship(relationship_target = DialoguePanels)]
struct DialoguePanelOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = DialoguePanelOf, linked_spawn)]
struct DialoguePanels(Vec<Entity>);

#[derive(Resource)]
pub(super) struct CourtAssets {
    dialogue: Handle<RepliqueDialogue>,
    font: Handle<Font>,
}

impl FromWorld for CourtAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            dialogue: assets.load("dialogue/court.rep"),
            font: assets.load("fonts/CourtDialogueSans.otf"),
        }
    }
}

pub(super) struct CourtPlugin;

impl Plugin for CourtPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RepliquePlugin)
            .init_resource::<CourtAssets>()
            .add_dialogue_command(heard)
            .add_dialogue_command(record_verdict)
            .add_dialogue_function(ready_for_verdict)
            .add_observer(respond)
            .add_systems(Update, (prepare_hearing, continue_dialogue).chain())
            .add_systems(PostUpdate, show_dialogue.after(DialogueSystem::Commands));
    }
}

fn prepare_hearing(
    mut commands: Commands,
    assets: Res<CourtAssets>,
    courts: Query<Entity, (With<CourtHearing>, Without<DialogueRunner>)>,
) {
    for court in &courts {
        commands
            .entity(court)
            .insert(DialogueRunner::new(assets.dialogue.clone()));
    }
}

type DialoguePlayers<'w, 's> = Query<
    'w,
    's,
    (&'static ChildOf, &'static mut LinearVelocity),
    (With<Player>, Without<DialogueLocked>, Without<Stunned>),
>;

#[derive(SystemParam)]
pub(super) struct CourtParticipants<'w, 's> {
    npcs: Query<'w, 's, (&'static CourtNpc, &'static ChildOf, &'static CollidingEntities)>,
    courts: Query<'w, 's, &'static mut CourtHearing, With<DialogueRunner>>,
    players: DialoguePlayers<'w, 's>,
}

pub(super) fn begin_dialogue(
    request: On<InteractRequested>,
    mut commands: Commands,
    assets: Res<CourtAssets>,
    dialogues: Res<Assets<RepliqueDialogue>>,
    mut participants: CourtParticipants,
    mut start: MessageWriter<StartDialogue>,
) {
    let Ok((npc, parent, collisions)) = participants.npcs.get(request.entity) else {
        return;
    };
    let Ok((player_parent, mut velocity)) = participants.players.get_mut(request.player) else {
        return;
    };
    let Ok(mut hearing) = participants.courts.get_mut(parent.parent()) else {
        return;
    };
    if player_parent.parent() != parent.parent()
        || !collisions.contains(&request.player)
        || hearing.player.is_some()
        || hearing.verdict.is_some()
        || !dialogues.contains(&assets.dialogue)
    {
        return;
    }

    velocity.x = 0.0;
    hearing.player = Some(request.player);
    commands.entity(request.player).insert(DialogueLocked);
    start.write(StartDialogue {
        runner: parent.parent(),
        node: npc.node().into(),
    });
}

fn continue_dialogue(
    mut commands: Commands,
    courts: Query<(Entity, &CourtHearing)>,
    players: Query<&ActionState<PlayerAction>, With<DialogueLocked>>,
) {
    for (court, hearing) in &courts {
        if let Some(Waiting::Line(token)) = hearing.waiting
            && let Some(player) = hearing.player
            && let Ok(actions) = players.get(player)
            && actions.just_pressed(&PlayerAction::Interact)
        {
            commands.trigger(DialogueResponse {
                entity: court,
                token,
                input: ResumeInput::Advance,
            });
        }
    }
}

fn respond(
    response: On<DialogueResponse>,
    mut commands: Commands,
    mut courts: Query<&mut CourtHearing>,
    mut resume: MessageWriter<ResumeDialogue>,
) {
    let Ok(mut hearing) = courts.get_mut(response.entity) else {
        return;
    };
    let valid = match (hearing.waiting, &response.input) {
        (Some(Waiting::Line(token)), ResumeInput::Advance)
        | (Some(Waiting::Choices(token)), ResumeInput::Select(_)) => token == response.token,
        _ => false,
    };
    if !valid || hearing.player.is_none() {
        return;
    }
    hearing.waiting = None;
    if let Some(panel) = hearing.panel.take() {
        commands.entity(panel).despawn();
    }
    resume.write(ResumeDialogue {
        token: response.token,
        input: response.input.clone(),
    });
}

fn show_dialogue(
    mut commands: Commands,
    assets: Res<CourtAssets>,
    mut lines: MessageReader<DialogueLine>,
    mut choices: MessageReader<DialogueChoices>,
    mut finished: MessageReader<DialogueFinished>,
    mut courts: Query<&mut CourtHearing>,
) {
    for line in lines.read() {
        let runner = line.token.runner();
        let Ok(mut hearing) = courts.get_mut(runner) else {
            continue;
        };
        hearing.waiting = Some(Waiting::Line(line.token));
        hearing.panel = Some(
            commands
                .spawn_scene(dialogue_box_scene(
                    assets.font.clone(),
                    line.speaker.as_deref().unwrap_or_default(),
                    &line.text,
                    line.token,
                    None,
                ))
                .insert(DialoguePanelOf(runner))
                .id(),
        );
    }
    for group in choices.read() {
        let runner = group.token.runner();
        let Ok(mut hearing) = courts.get_mut(runner) else {
            continue;
        };
        hearing.waiting = Some(Waiting::Choices(group.token));
        hearing.panel = Some(
            commands
                .spawn_scene(dialogue_box_scene(
                    assets.font.clone(),
                    "",
                    "",
                    group.token,
                    Some(&group.choices),
                ))
                .insert(DialoguePanelOf(runner))
                .id(),
        );
    }
    for event in finished.read() {
        if let Ok(mut hearing) = courts.get_mut(event.runner) {
            hearing.waiting = None;
            if let Some(panel) = hearing.panel.take() {
                commands.entity(panel).despawn();
            }
            if let Some(player) = hearing.player.take() {
                commands.entity(player).remove::<DialogueLocked>();
            }
        }
    }
}

#[replique_command]
fn heard(In(prosecution): In<bool>, call: DialogueCall, mut courts: Query<&mut CourtHearing>) {
    if let Ok(mut hearing) = courts.get_mut(call.runner()) {
        hearing.heard[usize::from(!prosecution)] = true;
    }
}

#[replique_function]
fn ready_for_verdict(In(()): In<()>, call: DialogueCall, courts: Query<&CourtHearing>) -> bool {
    courts
        .get(call.runner())
        .is_ok_and(|hearing| hearing.heard == [true; 2] && hearing.verdict.is_none())
}

#[replique_command]
fn record_verdict(In(guilty): In<bool>, call: DialogueCall, mut courts: Query<&mut CourtHearing>) {
    if let Ok(mut hearing) = courts.get_mut(call.runner())
        && hearing.heard == [true; 2]
        && hearing.verdict.is_none()
    {
        let verdict = if guilty {
            Verdict::Guilty
        } else {
            Verdict::NotGuilty
        };
        hearing.verdict = Some(verdict);
        info!("Court verdict: {verdict:?}");
    }
}
