#[cfg(test)]
mod tests;
use crate::config::{GameSettings, GameState};
use bevy::prelude::*;

#[derive(Resource, Debug)]
pub struct GameCalendar {
    pub total_days: u32,
    pub elapsed_days: u32,
}

impl GameCalendar {
    /// Monday is 1 and Sunday is 7; a new game starts on Saturday.
    pub fn weekday(&self) -> u32 {
        (self.elapsed_days % 7 + 5) % 7 + 1
    }

    pub fn remaining_days(&self) -> u32 {
        self.total_days.saturating_sub(self.elapsed_days)
    }

    pub fn is_weekend(&self) -> bool {
        self.weekday() >= 6
    }

    pub fn is_finished(&self) -> bool {
        self.elapsed_days >= self.total_days
    }
}

#[derive(Resource, Debug, Default)]
pub struct CommuteTime {
    pub elapsed: f32,
}

impl CommuteTime {
    pub fn remaining(&self, limit: f32) -> f32 {
        limit - self.elapsed
    }
}

#[derive(Resource, Debug)]
pub struct Money(pub i32);

impl FromWorld for Money {
    fn from_world(world: &mut World) -> Self {
        Self(world.resource::<GameSettings>().initial_money)
    }
}

#[derive(Resource, Debug, Default)]
pub struct HasHelmet(pub bool);

#[derive(Resource, Debug)]
pub struct Stamina(pub f32);

impl Stamina {
    pub const MAX: f32 = 100.0;
}

impl Default for Stamina {
    fn default() -> Self {
        Self(Self::MAX)
    }
}

#[derive(States, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CommuteState {
    #[default]
    Running,
    Paused,
}

#[derive(Component)]
struct CommuteHud;

#[derive(Component)]
struct CommuteHudText;

#[derive(Component)]
struct CalendarHud;

#[derive(Component)]
struct CalendarHudText;

#[derive(Component)]
struct StaminaBar;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<CommuteState>()
            .add_systems(
                OnEnter(GameState::Playing),
                (initialize_game, setup_hud).chain(),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_hud)
            .add_systems(
                Update,
                (
                    tick_commute.run_if(in_state(CommuteState::Running)),
                    update_hud.run_if(
                        resource_exists_and_changed::<CommuteTime>
                            .or_eager(resource_exists_and_changed::<Money>)
                            .or_eager(resource_exists_and_changed::<Stamina>)
                            .or_eager(resource_exists_and_changed::<GameSettings>),
                    ),
                    update_calendar_hud
                        .run_if(resource_exists_and_changed::<GameCalendar>),
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn initialize_game(world: &mut World) {
    let total_days = world.resource::<GameSettings>().total_days;
    world.insert_resource(GameCalendar {
        total_days,
        elapsed_days: 0,
    });
    world.insert_resource(CommuteTime::default());
    let money = Money::from_world(world);
    world.insert_resource(money);
    world.insert_resource(Stamina::default());
    world.insert_resource(HasHelmet::default());
    world
        .resource_mut::<NextState<CommuteState>>()
        .set(CommuteState::Running);
}

fn tick_commute(time: Res<Time>, mut commute: ResMut<CommuteTime>) {
    commute.elapsed += time.delta_secs();
}

fn setup_hud(mut commands: Commands) {
    commands.spawn((
        CalendarHud,
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            left: px(16),
            padding: UiRect::all(px(12)),
            border_radius: BorderRadius::all(px(6)),
            ..default()
        },
        GlobalZIndex(200),
        BackgroundColor(Color::srgb(0.12, 0.14, 0.2)),
        Pickable::IGNORE,
        children![(
            CalendarHudText,
            Text::default(),
            TextFont {
                font_size: FontSize::Px(22.0),
                ..default()
            },
            TextColor(Color::srgb(0.95, 0.96, 1.0)),
            Pickable::IGNORE,
        )],
    ));
    commands.spawn((
        CommuteHud,
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            right: px(16),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            padding: UiRect::all(px(12)),
            border_radius: BorderRadius::all(px(6)),
            ..default()
        },
        GlobalZIndex(200),
        BackgroundColor(Color::srgb(0.12, 0.14, 0.2)),
        Pickable::IGNORE,
        children![
            (
                CommuteHudText,
                Text::default(),
                TextFont {
                    font_size: FontSize::Px(22.0),
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.96, 1.0)),
                Pickable::IGNORE,
            ),
            (
                Node {
                    width: px(200),
                    height: px(12),
                    border_radius: BorderRadius::all(px(6)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.04, 0.05, 0.08)),
                Pickable::IGNORE,
                children![(
                    StaminaBar,
                    Node {
                        width: percent(100),
                        height: percent(100),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.8, 0.45)),
                    Pickable::IGNORE,
                )],
            ),
        ],
    ));
}

fn cleanup_hud(
    mut commands: Commands,
    roots: Query<Entity, Or<(With<CommuteHud>, With<CalendarHud>)>>,
) {
    for root in &roots {
        commands.entity(root).despawn();
    }
}

fn update_calendar_hud(
    calendar: Res<GameCalendar>,
    mut hud: Query<&mut Text, With<CalendarHudText>>,
) {
    let weekdays = [
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
        "Sunday",
    ];
    for mut text in &mut hud {
        text.0 = format!(
            "Days left: {}\n{}{}",
            calendar.remaining_days(),
            weekdays[(calendar.weekday() - 1) as usize],
            if calendar.is_weekend() {
                " (Weekend)"
            } else {
                ""
            },
        );
    }
}

fn update_hud(
    settings: Res<GameSettings>,
    commute: Res<CommuteTime>,
    money: Res<Money>,
    stamina: Res<Stamina>,
    mut hud: Query<&mut Text, With<CommuteHudText>>,
    mut bars: Query<&mut Node, With<StaminaBar>>,
) {
    for mut text in &mut hud {
        text.0 = format!(
            "Time: {:.1}s\nMoney: {}\nStamina: {:.0}/{:.0}",
            commute.remaining(settings.commute_limit),
            money.0,
            stamina.0,
            Stamina::MAX,
        );
    }
    for mut node in &mut bars {
        node.width = percent((stamina.0 / Stamina::MAX).clamp(0.0, 1.0) * 100.0);
    }
}
