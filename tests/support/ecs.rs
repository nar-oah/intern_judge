use bevy::prelude::*;

pub(crate) fn single<C: Component>(app: &mut App) -> Entity {
    let world = app.world_mut();
    world.query_filtered::<Entity, With<C>>().single(world).unwrap()
}

pub(crate) fn count<C: Component>(app: &mut App) -> usize {
    let world = app.world_mut();
    world.query_filtered::<Entity, With<C>>().iter(world).count()
}

pub(crate) fn assert_absent<C: Component>(app: &mut App) {
    assert_eq!(count::<C>(app), 0, "unexpected {} entity", std::any::type_name::<C>());
}

pub(crate) fn named(app: &mut App, expected: &str) -> Entity {
    let world = app.world_mut();
    world.query::<(Entity, &Name)>().iter(world)
        .find_map(|(entity, name)| (name.as_str() == expected).then_some(entity))
        .unwrap_or_else(|| panic!("missing entity named {expected:?}"))
}

pub(crate) fn text<C: Component>(app: &mut App) -> String {
    let world = app.world_mut();
    world.query_filtered::<&Text, With<C>>().single(world).unwrap().0.clone()
}

pub(crate) fn scene_entities(world: &World, root: Entity) -> Vec<Entity> {
    let mut entities = vec![root];
    let mut index = 0;
    while index < entities.len() {
        if let Some(children) = world.get::<Children>(entities[index]) {
            entities.extend(children.iter());
        }
        index += 1;
    }
    entities
}

pub(crate) fn assert_removed(app: &App, entities: &[Entity]) {
    for &entity in entities {
        assert!(app.world().get_entity(entity).is_err(), "stale scene entity {entity:?}");
    }
}
