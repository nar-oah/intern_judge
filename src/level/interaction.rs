use bevy::prelude::*;

use crate::player::{Player, SpeedBoost};

// BSN scenes provide this component; one system handles the E key for all.
#[derive(Component, Clone, Default)]
pub(super) enum Interactable {
    #[default]
    ConvenienceStore,
    Restaurant { worked: bool },
    TransitionPoint,
}

// Temporary demonstration money, not the game's final economy system.
#[derive(Resource, Default)]
pub(super) struct DemoWallet(pub u32);

const INTERACTION_DISTANCE: f32 = 90.0;

pub(super) fn interact(
    keys: Res<ButtonInput<KeyCode>>,
    player: Single<(Entity, &GlobalTransform), With<Player>>,
    mut targets: Query<(Entity, &GlobalTransform, &mut Interactable)>,
    mut commands: Commands,
    mut wallet: ResMut<DemoWallet>,
) {
    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }

    let (player_entity, player_transform) = *player;
    let position = player_transform.translation().truncate();

    // When several targets are in range, only the nearest receives the E key.
    let nearest = targets
        .iter_mut()
        .filter_map(|(entity, transform, _)| {
            let distance = position.distance_squared(transform.translation().truncate());
            (distance <= INTERACTION_DISTANCE * INTERACTION_DISTANCE).then_some((entity, distance))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(entity, _)| entity);

    let Some(entity) = nearest else {
        return;
    };
    let Ok((_, _, mut target)) = targets.get_mut(entity) else {
        return;
    };

    match &mut *target {
        Interactable::ConvenienceStore => {
            commands
                .entity(player_entity)
                .insert(SpeedBoost(Timer::from_seconds(5.0, TimerMode::Once)));
            info!("便利店：获得 5 秒移动加速（原型中免费）");
        }
        Interactable::Restaurant { worked } => {
            if !*worked {
                wallet.0 += 10;
                *worked = true;
                info!("餐厅：兼职收入 +10，当前演示现金 {}", wallet.0);
            } else {
                info!("餐厅：本次演示已领取兼职收入");
            }
        }
        Interactable::TransitionPoint => {
            info!("切换点：按 E 成功！暂不切换路段或法院");
        }
    }
}
