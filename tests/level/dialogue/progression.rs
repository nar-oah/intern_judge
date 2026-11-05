use super::*;

#[test]
fn consecutive_lines_each_need_e_and_available_buttons_keep_original_indices() {
    let (mut app, player, _) = court_app();
    let source = "\
:= start
第一句
第二句
第三句
-> 不可用 [if false]
    >> verdict(false)
    => END
-> 继续
    已选择
    >> verdict(true)
    => END
---
";
    start_fixture(&mut app, player, source);

    for expected in ["第一句", "第二句", "第三句"] {
        assert_prompt(&mut app, expected);
        advance(&mut app, 4);
        assert_prompt(&mut app, expected);
        assert!(buttons(&mut app).is_empty());
        press_e(&mut app);
    }
    wait_for_dialogue(&mut app, true);
    let choices = buttons(&mut app);
    assert_eq!(choices.len(), 1);
    assert_eq!((&choices[0].0, choices[0].1.as_str()), (&1, "继续"));
    let old_button = choices[0].2;
    let old_label = app.world().get::<Children>(old_button).unwrap()[0];
    select(&mut app, 1);
    wait_for_dialogue(&mut app, false);
    assert_prompt(&mut app, "已选择");
    assert!(buttons(&mut app).is_empty());
    assert!(app.world().get_entity(old_button).is_err());
    assert!(app.world().get_entity(old_label).is_err());
    assert!(app.world().resource::<CommandsRun>().0.is_empty());
    advance(&mut app, 4);
    assert_prompt(&mut app, "已选择");

    press_e(&mut app);
    advance(&mut app, 4);
    assert_closed(&mut app, player);
    let commands = &app.world().resource::<CommandsRun>().0;
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].args, [Value::Bool(true)]);
}
