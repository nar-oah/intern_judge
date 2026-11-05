use super::*;
use bevy::platform::time::Instant;

pub(super) const WALK_SPEED: f32 = 200.0;
pub(super) const JUMP_SPEED: f32 = 400.0;

pub(super) struct Movement {
    pub(super) world: World,
    pub(super) player: Entity,
}

impl Movement {
    pub(super) fn new(stamina: f32) -> Self {
        let mut world = World::new();
        world.insert_resource(Time::<Fixed>::default());
        world.insert_resource(GameSettings {
            multiplier: 2.5,
            ..crate::test_support::game_settings()
        });
        world.insert_resource(Stamina(stamina));
        let mut schedule = Schedule::new(FixedUpdate);
        schedule.add_systems((tick_stun, player_movement).chain());
        world.add_schedule(schedule);
        let player = world
            .spawn((
                Player {
                    move_speed: WALK_SPEED,
                    jump_speed: JUMP_SPEED,
                },
                ActionState::<PlayerAction>::default(),
                ShapeHits::default(),
                LinearVelocity(Vec2::new(0.0, -70.0)),
            ))
            .id();
        Self { world, player }
    }

    pub(super) fn actions(&mut self, pressed: &[PlayerAction]) {
        let mut actions = ActionState::default();
        for action in pressed {
            actions.press(action);
        }
        self.world.entity_mut(self.player).insert(actions);
    }

    pub(super) fn contacts(&mut self, surface_normals: &[Vec2]) {
        let hits = surface_normals
            .iter()
            .map(|&normal| ShapeHitData {
                entity: Entity::PLACEHOLDER,
                distance: 0.0,
                point1: Vec2::ZERO,
                point2: Vec2::ZERO,
                normal1: normal,
                normal2: -normal,
            })
            .collect();
        self.world.entity_mut(self.player).insert(ShapeHits(hits));
    }

    pub(super) fn step(&mut self, seconds: f32) {
        self.world
            .resource_mut::<Time<Fixed>>()
            .advance_by(Duration::from_secs_f32(seconds));
        self.world.run_schedule(FixedUpdate);
        let now = Instant::now();
        self.world
            .get_mut::<ActionState<PlayerAction>>(self.player)
            .unwrap()
            .tick(now, now);
    }

    pub(super) fn velocity(&self) -> Vec2 {
        self.world.get::<LinearVelocity>(self.player).unwrap().0
    }

    pub(super) fn stamina(&self) -> f32 {
        self.world.resource::<Stamina>().0
    }
}
