use super::*;
use crate::level::{RoadId, dialogue_ui::DialogueButton, interaction, scene};
use bevy::{
    app::{HierarchyPropagatePlugin, PropagateSet},
    asset::AssetPlugin,
    camera::{ComputedCameraValues, RenderTargetInfo},
    platform::time::Instant as InputInstant,
    scene::ScenePlugin,
    text::{FontCx, FontLoader},
    ui::{
        sync_font_size_to_em_size, ui_layout_system, ui_surface::UiSurface,
        update::propagate_ui_target_cameras,
    },
    ui_widgets::Activate,
};
use std::time::{Duration, Instant};

#[derive(Resource, Default)]
struct PresentedChoices(Vec<DialogueChoice>);

fn collect_choices(
    mut messages: MessageReader<DialogueChoices>,
    mut presented: ResMut<PresentedChoices>,
) {
    for group in messages.read() {
        presented.0 = group.choices.clone();
    }
}

struct Hearing {
    app: App,
    camera: Entity,
    court: Entity,
    player: Entity,
    npcs: [Entity; 2],
}

impl Hearing {
    fn new() -> Self {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin {
                file_path: format!("{}/assets", env!("CARGO_MANIFEST_DIR")),
                ..default()
            },
            ScenePlugin,
        ))
        .init_asset::<Font>()
        .init_asset_loader::<FontLoader>()
        .init_resource::<PresentedChoices>()
        .add_plugins(CourtPlugin)
        .add_systems(
            PostUpdate,
            interaction::interact.before(DialogueSystem::Runner),
        )
        .add_systems(PostUpdate, collect_choices.after(DialogueSystem::Commands));
        let camera = setup_ui_layout(&mut app);
        let (court, player, npcs) = spawn_court(app.world_mut());
        app.finish();
        app.cleanup();
        let mut hearing = Self {
            app,
            camera,
            court,
            player,
            npcs,
        };
        hearing.wait_until(|world| {
            let assets = world.resource::<CourtAssets>();
            world
                .resource::<Assets<RepliqueDialogue>>()
                .contains(&assets.dialogue)
                && world.resource::<Assets<Font>>().contains(&assets.font)
                && world.get::<DialogueRunner>(court).is_some()
        });
        hearing
    }

    fn frame(&mut self) {
        self.app.update();
        let now = InputInstant::now();
        let world = self.app.world_mut();
        for mut actions in world
            .query::<&mut ActionState<PlayerAction>>()
            .iter_mut(world)
        {
            actions.tick(now, now);
        }
    }

    fn wait_until(&mut self, condition: impl Fn(&World) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !condition(self.app.world()) {
            assert!(Instant::now() < deadline, "court dialogue did not progress");
            self.frame();
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    fn press_e(&mut self) {
        self.app
            .world_mut()
            .get_mut::<ActionState<PlayerAction>>(self.player)
            .unwrap()
            .release(&PlayerAction::Interact);
        self.frame();
        self.app
            .world_mut()
            .get_mut::<ActionState<PlayerAction>>(self.player)
            .unwrap()
            .press(&PlayerAction::Interact);
        self.frame();
    }

    fn start(&mut self, npc: CourtNpc) {
        for entity in self.npcs {
            let world = self.app.world_mut();
            let matches = world.get::<CourtNpc>(entity) == Some(&npc);
            let mut collisions = world.get_mut::<CollidingEntities>(entity).unwrap();
            collisions.clear();
            if matches {
                collisions.insert(self.player);
            }
        }
        self.app
            .world_mut()
            .get_mut::<LinearVelocity>(self.player)
            .unwrap()
            .x = 100.0;
        self.press_e();
        let court = self.court;
        self.wait_until(|world| {
            matches!(
                world.get::<CourtHearing>(court).unwrap().waiting,
                Some(Waiting::Line(_))
            )
        });
        assert!(
            self.app
                .world()
                .get::<DialogueLocked>(self.player)
                .is_some()
        );
        assert_eq!(
            self.app
                .world()
                .get::<LinearVelocity>(self.player)
                .unwrap()
                .x,
            0.0
        );
        let panel = self.state().panel.unwrap();
        assert_eq!(
            self.app.world().get::<DialoguePanelOf>(panel).unwrap().0,
            court
        );
        self.assert_panel_layout();
        let expected = match npc {
            CourtNpc::Prosecution => ("公诉方", "我们认为被告有罪，请法官依法判决。"),
            CourtNpc::Defense => ("辩护方", "我们认为证据不足，请法官判决无罪。"),
        };
        let texts = panel_texts(self.app.world_mut(), panel);
        assert!(texts.iter().any(|text| text == expected.0));
        assert!(texts.iter().any(|text| text == expected.1));
    }

    fn finish_line(&mut self) {
        self.press_e();
        let court = self.court;
        self.wait_until(|world| {
            let hearing = world.get::<CourtHearing>(court).unwrap();
            hearing.player.is_none() || matches!(hearing.waiting, Some(Waiting::Choices(_)))
        });
    }

    fn state(&self) -> &CourtHearing {
        self.app.world().get::<CourtHearing>(self.court).unwrap()
    }

    fn assert_panel_layout(&mut self) {
        let panel = self.state().panel.unwrap();
        let world = self.app.world_mut();
        let camera = world.get::<ComputedUiTargetCamera>(panel).unwrap();
        assert_eq!(
            camera.get(),
            Some(self.camera),
            "dialogue must reach the UI camera"
        );
        let size = world.get::<ComputedNode>(panel).unwrap().size();
        assert!(
            size.x > 1000.0 && size.y > 0.0,
            "dialogue must have a visible viewport layout: {size:?}"
        );
        for (entity, _) in world.query::<(Entity, &DialogueButton)>().iter(world) {
            assert_eq!(
                world.get::<ComputedUiTargetCamera>(entity).unwrap().get(),
                Some(self.camera)
            );
            let size = world.get::<ComputedNode>(entity).unwrap().size();
            assert!(
                size.x > 0.0 && size.y > 0.0,
                "dialogue button must be laid out: {size:?}"
            );
        }
    }

    fn assert_closed(&mut self) {
        assert!(self.state().player.is_none());
        assert!(self.state().panel.is_none());
        assert!(self.state().waiting.is_none());
        assert!(
            self.app
                .world()
                .get::<DialogueLocked>(self.player)
                .is_none()
        );
        let world = self.app.world_mut();
        assert_eq!(world.query::<&DialogueButton>().iter(world).count(), 0);
    }

    fn choose(&mut self, choice: usize) {
        let world = self.app.world_mut();
        let button = world
            .query::<(Entity, &DialogueButton)>()
            .iter(world)
            .find_map(|(entity, button)| (button.choice == Some(choice)).then_some(entity))
            .expect("the verdict is an available UI button");
        world.trigger(Activate { entity: button });
        let court = self.court;
        self.wait_until(|world| world.get::<CourtHearing>(court).unwrap().player.is_none());
    }
}

fn setup_ui_layout(app: &mut App) -> Entity {
    // Exercise Bevy's actual root discovery, camera propagation and layout without a window or GPU.
    app.init_resource::<UiScale>()
        .init_resource::<UiSurface>()
        .init_resource::<FontCx>()
        .init_resource::<RemSize>()
        .add_plugins((
            HierarchyPropagatePlugin::<ComputedUiTargetCamera>::new(PostUpdate),
            HierarchyPropagatePlugin::<ComputedUiRenderTargetInfo>::new(PostUpdate),
        ))
        .configure_sets(
            PostUpdate,
            PropagateSet::<ComputedUiTargetCamera>::default()
                .after(propagate_ui_target_cameras)
                .before(ui_layout_system),
        )
        .configure_sets(
            PostUpdate,
            PropagateSet::<ComputedUiRenderTargetInfo>::default()
                .after(propagate_ui_target_cameras)
                .before(ui_layout_system),
        )
        .add_systems(
            PostUpdate,
            (
                propagate_ui_target_cameras,
                sync_font_size_to_em_size,
                ui_layout_system,
            )
                .chain()
                .after(show_dialogue),
        );
    app.world_mut()
        .spawn((
            Camera2d,
            Camera {
                computed: ComputedCameraValues {
                    target_info: Some(RenderTargetInfo {
                        physical_size: UVec2::new(1280, 720),
                        scale_factor: 1.0,
                    }),
                    ..default()
                },
                ..default()
            },
        ))
        .id()
}

fn spawn_court(world: &mut World) -> (Entity, Entity, [Entity; 2]) {
    let court = world
        .spawn_scene(scene::road_scene(RoadId::Court, &[]))
        .unwrap()
        .id();
    let player = world
        .query_filtered::<(Entity, &ChildOf), With<Player>>()
        .iter(world)
        .find_map(|(entity, parent)| (parent.parent() == court).then_some(entity))
        .unwrap();
    let mut npcs = [Entity::PLACEHOLDER; 2];
    for (entity, npc, parent) in world.query::<(Entity, &CourtNpc, &ChildOf)>().iter(world) {
        if parent.parent() == court {
            npcs[usize::from(*npc == CourtNpc::Defense)] = entity;
        }
    }
    assert!(!npcs.contains(&Entity::PLACEHOLDER));
    (court, player, npcs)
}

fn panel_texts(world: &mut World, panel: Entity) -> Vec<String> {
    world
        .query::<(Entity, &Text)>()
        .iter(world)
        .filter(|(entity, _)| {
            let mut ancestor = *entity;
            while let Some(parent) = world.get::<ChildOf>(ancestor) {
                ancestor = parent.parent();
                if ancestor == panel {
                    return true;
                }
            }
            false
        })
        .map(|(_, text)| text.0.clone())
        .collect()
}

#[test]
fn either_npc_order_unlocks_both_verdicts_and_each_verdict_finishes_the_hearing() {
    for (first, second, choice, expected) in [
        (CourtNpc::Prosecution, CourtNpc::Defense, 0, Verdict::Guilty),
        (
            CourtNpc::Defense,
            CourtNpc::Prosecution,
            1,
            Verdict::NotGuilty,
        ),
    ] {
        let mut hearing = Hearing::new();
        hearing.start(first);
        assert_eq!(hearing.state().heard, [false; 2]);
        hearing.finish_line();
        hearing.assert_closed();
        assert!(
            hearing
                .app
                .world()
                .resource::<PresentedChoices>()
                .0
                .is_empty()
        );

        hearing.start(second);
        assert!(
            hearing
                .app
                .world()
                .resource::<PresentedChoices>()
                .0
                .is_empty()
        );
        hearing.finish_line();
        assert_eq!(hearing.state().heard, [true; 2]);
        assert!(
            hearing
                .app
                .world()
                .get::<DialogueLocked>(hearing.player)
                .is_some()
        );
        let choices = &hearing.app.world().resource::<PresentedChoices>().0;
        assert_eq!(
            choices
                .iter()
                .map(|choice| (choice.index, choice.text.as_str(), choice.available))
                .collect::<Vec<_>>(),
            [(0, "判决有罪", true), (1, "判决无罪", true)]
        );
        let panel = hearing.state().panel.unwrap();
        hearing.assert_panel_layout();
        let texts = panel_texts(hearing.app.world_mut(), panel);
        assert!(texts.iter().any(|text| text == "判决有罪"));
        assert!(texts.iter().any(|text| text == "判决无罪"));

        // E advances lines only; it cannot select a verdict accidentally.
        hearing.press_e();
        assert!(matches!(hearing.state().waiting, Some(Waiting::Choices(_))));
        assert_eq!(hearing.state().verdict, None);
        hearing.choose(choice);
        assert_eq!(hearing.state().verdict, Some(expected));
        hearing.assert_closed();

        hearing.press_e();
        for _ in 0..4 {
            hearing.frame();
        }
        hearing.assert_closed();
        assert_eq!(hearing.state().verdict, Some(expected));
    }
}

#[test]
fn holding_e_does_not_skip_a_line_and_repeating_one_npc_does_not_unlock_verdicts() {
    let mut hearing = Hearing::new();
    hearing.start(CourtNpc::Prosecution);
    let panel = hearing.state().panel.unwrap();
    for _ in 0..8 {
        hearing.frame();
    }
    assert_eq!(hearing.state().panel, Some(panel));
    assert_eq!(hearing.state().heard, [false; 2]);
    hearing.finish_line();
    hearing.assert_closed();
    assert_eq!(hearing.state().heard, [true, false]);

    hearing.start(CourtNpc::Prosecution);
    hearing.finish_line();
    hearing.assert_closed();
    assert_eq!(hearing.state().heard, [true, false]);
    assert!(
        hearing
            .app
            .world()
            .resource::<PresentedChoices>()
            .0
            .is_empty()
    );
}

#[test]
fn e_requires_npc_overlap_before_starting_a_dialogue() {
    let mut hearing = Hearing::new();
    hearing.press_e();
    for _ in 0..4 {
        hearing.frame();
    }
    hearing.assert_closed();
    assert_eq!(hearing.state().heard, [false; 2]);

    hearing.start(CourtNpc::Defense);
    assert!(matches!(hearing.state().waiting, Some(Waiting::Line(_))));
}

#[test]
fn removing_the_court_during_verdict_selection_cleans_the_dialogue_and_resets_progress() {
    let mut hearing = Hearing::new();
    hearing.start(CourtNpc::Prosecution);
    hearing.finish_line();
    hearing.start(CourtNpc::Defense);
    hearing.finish_line();
    assert!(matches!(hearing.state().waiting, Some(Waiting::Choices(_))));
    let panel = hearing.state().panel.unwrap();
    let old_court = hearing.court;
    let old_player = hearing.player;
    hearing.app.world_mut().despawn(old_court);
    hearing.frame();
    let world = hearing.app.world_mut();
    assert!(world.get_entity(old_court).is_err());
    assert!(world.get_entity(old_player).is_err());
    assert!(world.get_entity(panel).is_err());
    assert_eq!(world.query::<&DialogueRunner>().iter(world).count(), 0);
    assert_eq!(world.query::<&DialogueButton>().iter(world).count(), 0);
    assert_eq!(world.query::<&DialogueLocked>().iter(world).count(), 0);

    let (court, player, npcs) = spawn_court(world);
    hearing.court = court;
    hearing.player = player;
    hearing.npcs = npcs;
    hearing.frame();
    assert_eq!(hearing.state().heard, [false; 2]);
    assert_eq!(hearing.state().verdict, None);
    hearing.assert_closed();
    hearing.start(CourtNpc::Defense);
    hearing.finish_line();
    hearing.assert_closed();
    assert_eq!(hearing.state().heard, [false, true]);
}
