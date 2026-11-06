use super::CommuteTime;
use bevy::prelude::*;

pub(super) fn tick_commute(time: Res<Time>, mut commute: ResMut<CommuteTime>) {
    commute.elapsed += time.delta_secs();
}
