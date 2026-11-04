use crate::config::GameSettings;
use bevy::prelude::*;

#[derive(Resource, Debug)]
pub struct GameCalendar {
    pub total_days: u32,
    pub elapsed_days: u32,
}

impl GameCalendar {
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

#[derive(Component)]
pub(super) struct CalendarHud;

#[derive(Component)]
pub(super) struct CalendarHudText;

pub(crate) fn initialize_calendar(mut commands: Commands, settings: Res<GameSettings>) {
    commands.insert_resource(GameCalendar {
        total_days: settings.total_days,
        elapsed_days: 0,
    });
}

pub(super) fn setup_calendar_hud(mut commands: Commands) {
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
}

pub(super) fn cleanup_calendar_hud(
    mut commands: Commands,
    roots: Query<Entity, With<CalendarHud>>,
) {
    for root in &roots {
        commands.entity(root).despawn();
    }
}

pub(super) fn update_calendar_hud(
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
