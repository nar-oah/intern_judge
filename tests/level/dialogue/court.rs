use super::*;

#[test]
fn each_court_button_runs_its_branch_and_closes_the_dialogue() {
    for (index, expected) in [(0, Some(true)), (1, Some(false)), (2, None)] {
        let mut app = road_app();
        app.insert_resource(InitialRoad(RoadId::MarketRoad))
            .insert_resource(CaseDefs(vec![case_def(
                "loose_sign",
                RoadEffect::PlaceholderA,
                RoadEffect::PlaceholderB,
            )]));
        app.world_mut()
            .run_system_cached(crate::level::day::start_day)
            .unwrap();
        let (player, bench) = enter_court(&mut app);
        let selected = app.world().resource::<CurrentCase>().0.clone().unwrap();
        assert_eq!(selected.road, RoadId::MarketRoad);
        let mut expected_effects = HashMap::from([
            (RoadId::Court, vec![RoadEffect::ConstructionDetour]),
            (RoadId::ExampleRoad, vec![RoadEffect::FallingObject]),
        ]);
        let effect = expected.map(|guilty| {
            if guilty {
                selected.case.guilty
            } else {
                selected.case.not_guilty
            }
        });

        app.world_mut().resource_mut::<RoadEffects>().0 = expected_effects.clone();
        approach(&mut app, player, bench);
        let elapsed_days = app.world().resource::<GameCalendar>().elapsed_days;
        press_e(&mut app);
        wait_for_dialogue(&mut app, false);
        assert_current_case(&app, &selected);
        press_e(&mut app);
        wait_for_dialogue(&mut app, true);
        assert_current_case(&app, &selected);
        select(&mut app, index);
        assert_closed(&mut app, player);

        let commands = &app.world().resource::<CommandsRun>().0;
        match expected {
            Some(result) => {
                assert_eq!(commands.len(), 1);
                assert_eq!(commands[0].token.runner(), player);
                assert_eq!(commands[0].name, "verdict");
                assert_eq!(commands[0].args, [Value::Bool(result)]);

                expected_effects.insert(selected.road, vec![effect.unwrap()]);
                assert!(app.world().resource::<CurrentCase>().0.is_none());
            }
            None => {
                assert!(commands.is_empty(), "leaving must not issue a verdict");
                assert_current_case(&app, &selected);
            }
        }
        assert_eq!(app.world().resource::<RoadEffects>().0, expected_effects);
        advance(&mut app, 4);
        assert_closed(&mut app, player);
        assert_eq!(
            app.world().resource::<GameCalendar>().elapsed_days,
            elapsed_days
        );
        if expected.is_some() {
            single::<DailyBill>(&mut app);
            assert_absent::<Player>(&mut app);
            press_e(&mut app);
            app.update();
        }
        assert_eq!(
            app.world().resource::<GameCalendar>().elapsed_days,
            elapsed_days + u32::from(expected.is_some())
        );
        if expected.is_some() {
            assert!(app.world().get_entity(player).is_err());
            assert!(app.world().get_entity(bench).is_err());
            single::<Player>(&mut app);
            let root = single::<SceneRoot>(&mut app);
            assert_eq!(app.world().get::<RoadId>(root), Some(&RoadId::MarketRoad));
        } else {
            assert!(app.world().get_entity(player).is_ok());
            assert!(app.world().get_entity(bench).is_ok());
        }
    }
}

#[test]
fn court_participants_use_nearby_e_dialogue_and_restore_movement() {
    for (prefix, name, expected) in [
        (
            "loose_sign",
            "Prosecution",
            "公诉方：店主明知招牌松动仍未维修，招牌坠落砸伤行人，请求判决有罪。",
        ),
        (
            "loose_sign",
            "Defense",
            "辩护方：招牌当天刚通过检修，事故由突发强风造成，请求判决无罪。",
        ),
        (
            "balcony_planter",
            "Prosecution",
            "公诉方：住户将花盆放在阳台外沿，花盆坠落损坏车辆，请求判决有罪。",
        ),
        (
            "balcony_planter",
            "Defense",
            "辩护方：花盆有固定支架，是维修人员拆除支架后未装回，请求判决无罪。",
        ),
    ] {
        let mut app = road_app();
        app.insert_resource(CaseDefs(vec![case_def(
            prefix,
            RoadEffect::FallingObject,
            RoadEffect::ConstructionDetour,
        )]));
        let (player, _) = enter_court(&mut app);
        let selected = app.world().resource::<CurrentCase>().0.clone().unwrap();
        let participant = named(&mut app, name);
        press_e(&mut app);
        assert_closed(&mut app, player);
        approach(&mut app, player, participant);
        assert_closed(&mut app, player);

        KeyCode::KeyD.press(app.world_mut());
        app.update();
        assert!(app.world().get::<LinearVelocity>(player).unwrap().x > 0.0);
        press_e(&mut app);
        wait_for_dialogue(&mut app, false);
        assert_prompt(&mut app, expected);
        assert_current_case(&app, &selected);
        advance(&mut app, 4);
        assert_prompt(&mut app, expected);
        assert_eq!(app.world().get::<LinearVelocity>(player).unwrap().x, 0.0);
        assert!(buttons(&mut app).is_empty());

        press_e(&mut app);
        advance(&mut app, 4);
        assert_closed(&mut app, player);
        assert!(app.world().get::<LinearVelocity>(player).unwrap().x > 0.0);
        assert!(app.world().resource::<CommandsRun>().0.is_empty());
        assert!(app.world().resource::<RoadEffects>().0.is_empty());
        assert_current_case(&app, &selected);
    }
}
