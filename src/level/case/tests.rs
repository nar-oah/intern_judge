use super::select_case;
use crate::{
    config::{CaseDef, CaseDefs},
    level::{RoadEffect, RoadEffects, RoadId, SelectedCase, TodayRoads},
};
use rand::{SeedableRng, rngs::StdRng};

fn case(prefix: &str, guilty: RoadEffect, not_guilty: RoadEffect) -> CaseDef {
    CaseDef {
        dialogue_prefix: prefix.into(),
        guilty,
        not_guilty,
    }
}

#[test]
fn either_outcome_effect_makes_a_case_unavailable() {
    let today = TodayRoads([RoadId::ExampleRoad, RoadId::MarketRoad].into());
    for (guilty, not_guilty) in [
        (RoadEffect::FallingObject, RoadEffect::ConstructionDetour),
        (RoadEffect::PlaceholderA, RoadEffect::PlaceholderB),
    ] {
        let cases = CaseDefs(vec![case("blocked", guilty, not_guilty)]);
        for effect in [guilty, not_guilty] {
            let effects = RoadEffects([(RoadId::ExampleRoad, vec![effect])].into());
            let mut rng = StdRng::seed_from_u64(17);
            for _ in 0..64 {
                assert_eq!(
                    select_case(&today, &effects, &cases, &mut rng),
                    SelectedCase {
                        road: RoadId::MarketRoad,
                        case: cases.0[0].clone(),
                    },
                    "existing {effect:?} must exclude the case on ExampleRoad",
                );
            }
        }
    }
}

#[test]
fn unrelated_effects_leave_the_case_available_and_selection_owns_its_definition() {
    let today = TodayRoads([RoadId::ExampleRoad].into());
    let effects = RoadEffects(
        [(
            RoadId::ExampleRoad,
            vec![RoadEffect::PlaceholderA, RoadEffect::PlaceholderB],
        )]
        .into(),
    );
    let expected = case(
        "available",
        RoadEffect::FallingObject,
        RoadEffect::ConstructionDetour,
    );
    let mut cases = CaseDefs(vec![expected.clone()]);
    let mut rng = StdRng::seed_from_u64(23);

    let selected = select_case(&today, &effects, &cases, &mut rng);
    cases.0[0].dialogue_prefix = "changed_after_selection".into();

    assert_eq!(
        selected,
        SelectedCase {
            road: RoadId::ExampleRoad,
            case: expected,
        },
    );
}

#[test]
fn unvisited_roads_and_roads_without_available_cases_are_excluded() {
    let today = TodayRoads([RoadId::ExampleRoad, RoadId::MarketRoad].into());
    let effects = RoadEffects(
        [
            (
                RoadId::ExampleRoad,
                vec![RoadEffect::FallingObject, RoadEffect::PlaceholderA],
            ),
            (RoadId::MarketRoad, vec![RoadEffect::ConstructionDetour]),
            (RoadId::EmptyRoad1, vec![]),
        ]
        .into(),
    );
    let cases = CaseDefs(vec![
        case(
            "blocked_on_both_roads",
            RoadEffect::FallingObject,
            RoadEffect::ConstructionDetour,
        ),
        case(
            "available_on_market",
            RoadEffect::PlaceholderA,
            RoadEffect::PlaceholderB,
        ),
    ]);
    let mut rng = StdRng::seed_from_u64(31);

    for _ in 0..128 {
        assert_eq!(
            select_case(&today, &effects, &cases, &mut rng),
            SelectedCase {
                road: RoadId::MarketRoad,
                case: cases.0[1].clone(),
            },
        );
    }
}

#[test]
fn selection_is_uniform_by_road_then_by_available_case() {
    let today = TodayRoads([RoadId::ExampleRoad, RoadId::MarketRoad].into());
    let effects = RoadEffects([(RoadId::ExampleRoad, vec![RoadEffect::FallingObject])].into());
    let cases = CaseDefs(vec![
        case(
            "first",
            RoadEffect::FallingObject,
            RoadEffect::ConstructionDetour,
        ),
        case(
            "second",
            RoadEffect::FallingObject,
            RoadEffect::PlaceholderA,
        ),
        case("third", RoadEffect::PlaceholderA, RoadEffect::PlaceholderB),
    ]);
    let mut rng = StdRng::seed_from_u64(41);
    let mut road_counts = [0_u32; 2];
    let mut market_case_counts = [0_u32; 3];

    for _ in 0..24_000 {
        let selected = select_case(&today, &effects, &cases, &mut rng);
        match selected.road {
            RoadId::ExampleRoad => {
                road_counts[0] += 1;
                assert_eq!(selected.case, cases.0[2]);
            }
            RoadId::MarketRoad => {
                road_counts[1] += 1;
                let index = cases
                    .0
                    .iter()
                    .position(|case| case == &selected.case)
                    .unwrap();
                market_case_counts[index] += 1;
            }
            road => panic!("unvisited road was selected: {road:?}"),
        }
    }

    // One road has one available case and the other has three. Flattening all
    // (road, case) pairs would give the roads probabilities of 1/4 and 3/4.
    // Wide margins also tolerate either iteration order of the HashSet.
    for count in road_counts {
        assert!((10_800..=13_200).contains(&count), "{road_counts:?}");
    }
    for count in market_case_counts {
        assert!((3_400..=4_600).contains(&count), "{market_case_counts:?}",);
    }
}
