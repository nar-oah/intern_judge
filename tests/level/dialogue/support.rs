use super::*;

#[derive(Resource, Default)]
pub(super) struct CommandsRun(pub(super) Vec<DialogueCommand>);

pub(super) fn road_app() -> App {
    let mut app = headless_app(Vec2::ZERO);
    app.insert_resource(InitialRoad(RoadId::ExampleRoad))
        .init_resource::<CommandsRun>()
        .add_plugins((GamePlugin, PlayerPlugin, LevelPlugin))
        .add_systems(
            PostUpdate,
            (|mut messages: MessageReader<DialogueCommand>, mut ran: ResMut<CommandsRun>| {
                ran.0.extend(messages.read().cloned());
            })
            .after(DialogueSystem::Commands),
        );
    finish_app(&mut app);
    app.world_mut().resource_mut::<GameCalendar>().elapsed_days = 2;
    crate::level::day::start_day(app.world_mut());
    app.update();
    app
}

pub(super) fn enter_court(app: &mut App) -> (Entity, Entity) {
    let point = single::<SwitchPoint>(app);
    open_destinations(app.world_mut(), point);
    let button = crate::level::tests::support::destination_button(app, RoadId::Court);
    select_destination(app.world_mut(), button);
    app.update();
    assert!(app.world().resource::<CurrentCase>().0.is_some());
    let player = single::<Player>(app);
    let bench = named(app, "JudgeBench");
    (player, bench)
}

pub(super) fn court_app() -> (App, Entity, Entity) {
    let mut app = road_app();
    let (player, bench) = enter_court(&mut app);
    (app, player, bench)
}

pub(super) fn assert_current_case(app: &App, expected: &SelectedCase) {
    let current = app.world().resource::<CurrentCase>().0.as_ref().unwrap();
    assert_eq!(current, expected);
}

pub(super) fn approach(app: &mut App, player: Entity, target: Entity) {
    let center = app
        .world()
        .get::<GlobalTransform>(target)
        .unwrap()
        .translation();
    let position = Vec2::new(center.x, -137.5);
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation = position.extend(0.0);
    app.world_mut().get_mut::<Position>(player).unwrap().0 = position;
    app.world_mut().get_mut::<LinearVelocity>(player).unwrap().0 = Vec2::ZERO;
    advance(app, 3);
    assert!(
        app.world()
            .get::<CollidingEntities>(target)
            .unwrap()
            .contains(&player)
    );
}

pub(super) fn wait_for_dialogue(app: &mut App, choices: bool) -> Entity {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let world = app.world_mut();
        if let Some(entity) = world
            .query::<(Entity, &DialogueBox)>()
            .iter(world)
            .find_map(|(entity, dialog)| {
                let ready = if choices {
                    dialog.choices.is_some()
                } else {
                    dialog.line.is_some()
                };
                ready.then_some(entity)
            })
        {
            return entity;
        }
        app.update();
        assert!(Instant::now() < deadline, "court dialogue did not load");
        std::thread::sleep(Duration::from_millis(1));
    }
}

pub(super) fn assert_prompt(app: &mut App, expected: &str) {
    assert_eq!(text::<DialoguePrompt>(app), expected);
}

pub(super) fn buttons(app: &mut App) -> Vec<(usize, String, Entity)> {
    let world = app.world_mut();
    let mut buttons = world
        .query::<(Entity, &DialogueChoiceButton, &Children)>()
        .iter(world)
        .map(|(entity, choice, children)| {
            assert!(world.get::<Button>(entity).is_some());
            let text = world.get::<Text>(children[0]).unwrap().0.clone();
            (choice.0, text, entity)
        })
        .collect::<Vec<_>>();
    buttons.sort_by_key(|(index, _, _)| *index);
    buttons
}

pub(super) fn select(app: &mut App, index: usize) {
    let button = buttons(app)
        .into_iter()
        .find(|(choice, _, _)| *choice == index)
        .unwrap()
        .2;
    app.world_mut().trigger(Activate { entity: button });
    app.world_mut().trigger(Activate { entity: button });
    advance(app, 4);
}

pub(super) fn assert_closed(app: &mut App, player: Entity) {
    assert!(app.world().get::<DialogueRunner>(player).is_none());
    assert_absent::<DialogueBox>(app);
    assert_absent::<DialoguePrompt>(app);
    assert_absent::<DialogueOptions>(app);
    assert_absent::<DialogueChoiceButton>(app);
}

pub(super) fn start_fixture(app: &mut App, runner: Entity, source: &str) {
    app.world()
        .resource::<EmbeddedAssetRegistry>()
        .insert_asset(
            "fixture.rep".into(),
            Path::new("fixture.rep"),
            source.as_bytes().to_vec(),
        );
    let dialogue = app
        .world()
        .resource::<AssetServer>()
        .load("embedded://fixture.rep");
    app.world_mut()
        .entity_mut(runner)
        .insert(DialogueRunner::new(dialogue));
    app.world_mut()
        .spawn_scene(ui::dialogue_box_scene())
        .unwrap();
    app.world_mut().write_message(StartDialogue {
        runner,
        node: "start".into(),
    });
    wait_for_dialogue(app, false);
}
