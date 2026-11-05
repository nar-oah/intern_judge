use super::*;

pub(super) fn game() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin));
    insert_config(&mut app);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        125,
    )))
    .add_plugins(GamePlugin);
    finish_app(&mut app);
    app
}

