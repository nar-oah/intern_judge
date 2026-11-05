use super::*;

#[derive(Resource, Default)]
struct OtherInteractions(usize);

#[test]
fn opening_and_holding_e_keeps_the_first_line_until_a_fresh_press() {
    let (mut app, player, bench) = court_app();
    press_e(&mut app);
    assert_closed(&mut app, player);
    approach(&mut app, player, bench);
    assert_closed(&mut app, player);

    press_e(&mut app);
    assert!(app.world().get::<DialogueRunner>(player).is_some());
    wait_for_dialogue(&mut app, false);
    advance(&mut app, 4);
    assert_prompt(&mut app, "请作出你的判决");
    assert!(buttons(&mut app).is_empty());

    press_e(&mut app);
    wait_for_dialogue(&mut app, true);
    assert_eq!(
        buttons(&mut app)
            .into_iter()
            .map(|(index, text, _)| (index, text))
            .collect::<Vec<_>>(),
        [(0, "有罪".into()), (1, "无罪".into()), (2, "离开".into())]
    );
    KeyCode::ArrowLeft.press(app.world_mut());
    KeyCode::ArrowRight.press(app.world_mut());
    press_e(&mut app);
    advance(&mut app, 4);
    assert_eq!(buttons(&mut app).len(), 3);
    assert!(app.world().resource::<CommandsRun>().0.is_empty());

    app.world_mut().despawn(player);
    app.world_mut().flush();
    app.update();
    assert_closed(&mut app, player);
    assert!(app.world().get_entity(bench).is_ok());
}

#[test]
fn dialogue_confirm_uses_the_ui_root_mapping_independently_of_player_input() {
    let (mut app, player, participant) = court_app();
    approach(&mut app, player, participant);
    press_e(&mut app);
    let dialog = wait_for_dialogue(&mut app, false);
    app.world_mut()
        .entity_mut(player)
        .remove::<InputMap<crate::player::PlayerAction>>()
        .remove::<ActionState<crate::player::PlayerAction>>();
    app.world_mut()
        .entity_mut(dialog)
        .insert(InputMap::new([(UiAction::Confirm, KeyCode::Enter)]));
    press_e(&mut app);
    advance(&mut app, 2);
    assert_prompt(&mut app, "请作出你的判决");
    assert!(buttons(&mut app).is_empty());

    press_key(&mut app, KeyCode::Enter);
    wait_for_dialogue(&mut app, true);
    assert_eq!(buttons(&mut app).len(), 3);
}

#[test]
fn dialogue_blocks_controls_and_other_buildings_then_restores_all_input() {
    let (mut app, player, bench) = court_app();
    app.init_resource::<OtherInteractions>();
    approach(&mut app, player, bench);
    assert!(
        app.world()
            .get::<ShapeHits>(player)
            .unwrap()
            .iter()
            .any(|hit| (-hit.normal2).dot(Vec2::Y) > 0.7)
    );

    KeyCode::KeyD.press(app.world_mut());
    KeyCode::ShiftLeft.press(app.world_mut());
    app.update();
    assert!(app.world().get::<LinearVelocity>(player).unwrap().x > 0.0);
    press_e(&mut app);
    let stamina = app.world().resource::<Stamina>().0;
    assert_eq!(app.world().get::<LinearVelocity>(player).unwrap().x, 0.0);
    KeyCode::Space.press(app.world_mut());
    wait_for_dialogue(&mut app, false);

    app.world_mut().entity_mut(bench).remove::<Interactable>();
    let other_building = app
        .world_mut()
        .spawn((
            Interactable,
            Collider::rectangle(240.0, 70.0),
            Sensor,
            CollidingEntities::default(),
            Transform::from_xyz(0.0, -127.5, -1.0),
        ))
        .observe(
            |_: On<PausedInteractRequested>, mut requests: ResMut<OtherInteractions>| {
                requests.0 += 1;
            },
        )
        .id();
    advance(&mut app, 4);
    assert!(
        app.world()
            .get::<CollidingEntities>(other_building)
            .unwrap()
            .contains(&player)
    );
    press_e(&mut app);
    wait_for_dialogue(&mut app, true);
    press_e(&mut app);
    advance(&mut app, 4);
    assert_eq!(
        app.world().get::<LinearVelocity>(player).unwrap().0,
        Vec2::ZERO
    );
    assert_eq!(app.world().resource::<Stamina>().0, stamina);
    assert_eq!(app.world().resource::<OtherInteractions>().0, 0);

    select(&mut app, 2);
    assert_closed(&mut app, player);
    advance(&mut app, 2);
    assert!(app.world().get::<LinearVelocity>(player).unwrap().x > 0.0);
    assert!(app.world().resource::<Stamina>().0 < stamina);
    press_e(&mut app);
    assert_eq!(app.world().resource::<OtherInteractions>().0, 1);

    press_key(&mut app, KeyCode::Space);
    assert!(app.world().get::<LinearVelocity>(player).unwrap().y > 0.0);
}
