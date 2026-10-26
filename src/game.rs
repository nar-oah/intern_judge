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

#[derive(Resource, Debug, Default)]
pub struct Money(pub i32);

#[derive(States, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CommuteState {
    #[default]
    Running,
    Paused,
}

#[derive(Component)]
struct CommuteHud;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CommuteTime>()
            .init_resource::<Money>()
            .init_state::<CommuteState>()
            .add_systems(Startup, setup_hud)
            .add_systems(
                Update,
                (
                    tick_commute.run_if(in_state(CommuteState::Running)),
                    update_hud,
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
            padding: UiRect::all(px(12)),
            border_radius: BorderRadius::all(px(6)),
            ..default()
        },
        GlobalZIndex(200),
        BackgroundColor(Color::srgb(0.12, 0.14, 0.2)),
        Text::default(),
        TextFont {
            font_size: FontSize::Px(22.0),
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.96, 1.0)),
        Pickable::IGNORE,
    ));
}

fn update_hud(
    commute: Res<CommuteTime>,
    money: Res<Money>,
    mut hud: Query<&mut Text, With<CommuteHud>>,
) {
    for mut text in &mut hud {
        text.0 = format!("Time: {:.1}s\nMoney: {}", commute.remaining(), money.0);
    }
}
