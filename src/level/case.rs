#[cfg(test)]
#[path = "../../tests/level/case/mod.rs"]
mod tests;

use super::{RoadEffects, SelectedCase, TodayRoads};
use crate::config::CaseDefs;
use rand::{Rng, seq::IndexedRandom};

pub(super) fn select_case(
    today: &TodayRoads,
    effects: &RoadEffects,
    cases: &CaseDefs,
    rng: &mut impl Rng,
) -> SelectedCase {
    let candidates = today
        .0
        .iter()
        .filter_map(|&road| {
            let effects = effects.0.get(&road).map_or(&[][..], Vec::as_slice);
            let available = cases
                .0
                .iter()
                .filter(|case| {
                    !effects.contains(&case.guilty) && !effects.contains(&case.not_guilty)
                })
                .collect::<Vec<_>>();
            (!available.is_empty()).then_some((road, available))
        })
        .collect::<Vec<_>>();
    let (road, available) = candidates.choose(rng).unwrap();
    SelectedCase {
        road: *road,
        case: (*available.choose(rng).unwrap()).clone(),
    }
}
