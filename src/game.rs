#[cfg(test)]
mod tests;
use bevy::prelude::*;

#[derive(Resource, Debug)]
pub struct CommuteTime {
    pub limit: f32,
    pub elapsed: f32,
}

impl Default for CommuteTime {
    fn default() -> Self {
        Self {
            limit: 300.0,
            elapsed: 0.0,
        }
    }
}

impl CommuteTime {
    pub fn remaining(&self) -> f32 {
        self.limit - self.elapsed
    }
}

#[derive(Resource, Debug)]
pub struct Money(pub i32);

impl Default for Money {
    fn default() -> Self {
        Self(100)
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
struct StaminaBar;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CommuteTime>()
            .init_resource::<Money>()
            .init_resource::<Stamina>()
            .init_resource::<HasHelmet>()
            .init_state::<CommuteState>()
            .add_systems(Startup, setup_hud)
            .add_systems(
                Update,
                (
                    tick_commute.run_if(in_state(CommuteState::Running)),
                    update_hud.run_if(
                        resource_changed::<CommuteTime>
                            .or_eager(resource_changed::<Money>)
                            .or_eager(resource_changed::<Stamina>),
                    ),
                )
                    .chain(),
            );
    }
}

fn tick_commute(time: Res<Time>, mut commute: ResMut<CommuteTime>) {
    commute.elapsed += time.delta_secs();
}

fn setup_hud(mut commands: Commands) {
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

fn update_hud(
    commute: Res<CommuteTime>,
    money: Res<Money>,
    stamina: Res<Stamina>,
    mut hud: Query<&mut Text, With<CommuteHudText>>,
    mut bars: Query<&mut Node, With<StaminaBar>>,
) {
    for mut text in &mut hud {
        text.0 = format!(
            "Time: {:.1}s\nMoney: {}\nStamina: {:.0}/{:.0}",
            commute.remaining(),
            money.0,
            stamina.0,
            Stamina::MAX,
        );
    }
    for mut node in &mut bars {
        node.width = percent((stamina.0 / Stamina::MAX).clamp(0.0, 1.0) * 100.0);
    }
}
