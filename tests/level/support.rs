use crate::{
    config::{GameSettings, RoadGraph},
    game::{CommuteState, GamePlugin, calendar::GameCalendar},
    level::{
        CurrentCase, LevelPlugin, RoadEffects, RoadId, RoadUnlocks, RunningInteractRequested,
        SceneRoot, TodayRoads,
        day::start_day,
        map::MapLayout,
        navigation,
        scene::{self, ending::InternshipEnded},
        ui::{TravelButtonAction, TravelUiRoot},
    },
    player::{Player, PlayerPlugin},
    test_support::{assert_absent, finish_app, headless_app, level_app, single},
};
use bevy::{prelude::*, ui_widgets::Activate};

pub(super) fn day_app(total_days: u32) -> App {
    configured_day_app(total_days, |_| {})
}

pub(super) fn configured_day_app(total_days: u32, configure: impl FnOnce(&mut App)) -> App {
    let mut app = headless_app(Vec2::ZERO);
    app.world_mut().resource_mut::<GameSettings>().total_days = total_days;
    configure(&mut app);
    app.add_plugins((GamePlugin, PlayerPlugin, LevelPlugin));
    finish_app(&mut app);
    app
}

pub(super) fn weekday_app(total_days: u32) -> App {
    let mut app = day_app(total_days);
    app.world_mut().resource_mut::<GameCalendar>().elapsed_days = 2;
    start_day(app.world_mut());
    app.update();
    app
}

pub(in crate::level) fn destination_button(app: &mut App, destination: RoadId) -> Entity {
    let world = app.world_mut();
    world
        .query::<(Entity, &TravelButtonAction)>()
        .iter(world)
        .find_map(|(entity, action)| (action.0 == destination).then_some(entity))
        .expect("the destination must be offered")
}

pub(in crate::level) fn activate(app: &mut App, destination: RoadId) {
    let button = destination_button(app, destination);
    app.world_mut().trigger(Activate { entity: button });
    app.world_mut().flush();
}

pub(super) fn assert_map(app: &mut App, choices: &[RoadId]) -> Entity {
    let root = single::<TravelUiRoot>(app);
    assert_eq!(app.world().get::<TravelUiRoot>(root).unwrap().0, choices);
    assert_absent::<Player>(app);
    root
}

pub(super) fn assert_road(app: &mut App, road: RoadId) -> (Entity, Entity) {
    let root = single::<SceneRoot>(app);
    let player = single::<Player>(app);
    assert_eq!(app.world().get::<RoadId>(root), Some(&road));
    assert_eq!(app.world().get::<ChildOf>(player).unwrap().parent(), root);
    assert_absent::<InternshipEnded>(app);
    assert_absent::<TravelUiRoot>(app);
    (root, player)
}

pub(super) fn assert_commute(app: &App, state: CommuteState) {
    assert_eq!(*app.world().resource::<State<CommuteState>>().get(), state);
}

pub(super) fn assert_today_roads(app: &App, roads: &[RoadId]) {
    assert_eq!(
        app.world().resource::<TodayRoads>().0,
        roads.iter().copied().collect()
    );
}

// Real scene ancestry and observers without advancing physics.
pub(super) fn navigation_app() -> App {
    let mut app = level_app();
    let layout = MapLayout::from_graph(app.world().resource::<RoadGraph>());
    let unlocks = app.world().resource::<RoadGraph>().initial_unlocks.clone();
    app.insert_resource(layout)
        .add_observer(scene::enter_road)
        .add_observer(navigation::open_unlock_map)
        .insert_resource(RoadUnlocks(unlocks))
        .init_resource::<RoadEffects>()
        .init_resource::<TodayRoads>()
        .init_resource::<CurrentCase>();
    app.add_plugins(GamePlugin);
    finish_app(&mut app);
    app.world_mut().resource_mut::<GameCalendar>().elapsed_days = 2;
    app.world_mut()
        .trigger(scene::EnterRoad(RoadId::ExampleRoad));
    app.world_mut().flush();
    app
}

pub(in crate::level) fn open_destinations(world: &mut World, point: Entity) {
    world.trigger(RunningInteractRequested { entity: point });
    world.flush();
}

pub(in crate::level) fn select_destination(world: &mut World, button: Entity) {
    world.trigger(Activate { entity: button });
    world.flush();
}

pub(in crate::level) fn open_unlock_map(world: &mut World) {
    world.trigger(navigation::OpenUnlockMap);
    world.flush();
}
