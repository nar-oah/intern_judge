use crate::{
    config::{
        BuildingValues, CaseDef, CaseDefs, ConvenienceKind, ConvenienceValues, GameSettings,
        GameState, LotteryPrize, LotteryValues, RoadGraph,
    },
    game::calendar::GameCalendar,
    level::{RoadEffect, RoadId},
};
use bevy::prelude::*;

pub(crate) fn insert_config(app: &mut App) {
    // Behavioral tests use their own data so balancing the shipped RON assets
    // does not change their expectations. Config tests exercise the real loader.
    app.insert_state(GameState::Playing)
        .insert_resource(CaseDefs(vec![case_def(
            "loose_sign",
            RoadEffect::FallingObject,
            RoadEffect::ConstructionDetour,
        )]))
        .insert_resource(game_settings())
        .insert_resource(RoadGraph {
            roads: [
                (RoadId::ExampleRoad, vec![RoadId::MarketRoad, RoadId::Court]),
                (RoadId::MarketRoad, vec![RoadId::Court]),
                (RoadId::Court, vec![]),
            ]
            .into(),
            initial_unlocks: [
                (RoadId::ExampleRoad, true),
                (RoadId::MarketRoad, true),
                (RoadId::Court, true),
            ]
            .into(),
        })
        .insert_resource(GameCalendar {
            total_days: 28,
            elapsed_days: 2,
        })
        .insert_resource(building_values());
}

pub(crate) fn game_settings() -> GameSettings {
    GameSettings {
        total_days: 28,
        commute_limit: 8.0,
        initial_money: 73,
        weekly_salary: 31,
        weekly_rent: 47,
        multiplier: 2.0,
        per_second: 8.0,
    }
}

pub(crate) fn building_values() -> BuildingValues {
    BuildingValues {
        cat_reward: 17,
        lottery: LotteryValues {
            price: 7,
            prizes: vec![LotteryPrize {
                reward: 13,
                weight: 1,
            }],
        },
        convenience_stores: [
            (
                ConvenienceKind::Small,
                ConvenienceValues {
                    price: 7,
                    stamina_recovery: 25.0,
                },
            ),
            (
                ConvenienceKind::Large,
                ConvenienceValues {
                    price: 13,
                    stamina_recovery: 55.0,
                },
            ),
        ]
        .into(),
    }
}

pub(crate) fn case_def(prefix: &str, guilty: RoadEffect, not_guilty: RoadEffect) -> CaseDef {
    CaseDef {
        dialogue_prefix: prefix.into(),
        guilty,
        not_guilty,
    }
}
