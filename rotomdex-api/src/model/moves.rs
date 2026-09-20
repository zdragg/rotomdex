use alloc::{string::String, vec::Vec};

use super::{
    contests::{ContestEffect, ContestType, SuperContestEffect},
    games::{Generation, VersionGroup},
    pokemon::{AbilityEffectChange, Pokemon, Stat, Type},
    resource::{
        ApiResource, Description, MachineVersionDetail, Name, NamedApiResource, VerboseEffect,
    },
    utility::Language,
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Move {
    pub id: i64,
    pub name: String,
    pub accuracy: Option<i64>,
    pub effect_chance: Option<i64>,
    pub pp: Option<i64>,
    pub priority: i64,
    pub power: Option<i64>,
    pub contest_combos: Option<ContestComboSets>,
    pub contest_type: Option<NamedApiResource<ContestType>>,
    pub contest_effect: Option<ApiResource<ContestEffect>>,
    pub damage_class: NamedApiResource<MoveDamageClass>,
    pub effect_entries: Vec<VerboseEffect>,
    pub effect_changes: Vec<AbilityEffectChange>,
    pub learned_by_pokemon: Vec<NamedApiResource<Pokemon>>,
    pub flavor_text_entries: Vec<MoveFlavorText>,
    pub generation: NamedApiResource<Generation>,
    pub machines: Vec<MachineVersionDetail>,
    pub meta: Option<MoveMetaData>,
    pub names: Vec<Name>,
    pub past_values: Vec<PastMoveStatValues>,
    pub stat_changes: Vec<MoveStatChange>,
    pub super_contest_effect: Option<ApiResource<SuperContestEffect>>,
    pub target: NamedApiResource<MoveTarget>,
    #[serde(rename = "type")]
    pub type_: NamedApiResource<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct ContestComboSets {
    pub normal: ContestComboDetail,
    #[serde(rename = "super")]
    pub super_: ContestComboDetail,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct ContestComboDetail {
    pub use_before: Option<Vec<NamedApiResource<Move>>>,
    pub use_after: Option<Vec<NamedApiResource<Move>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct MoveFlavorText {
    pub flavor_text: String,
    pub language: NamedApiResource<Language>,
    pub version_group: NamedApiResource<VersionGroup>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct MoveMetaData {
    pub ailment: NamedApiResource<MoveAilment>,
    pub category: NamedApiResource<MoveCategory>,
    pub min_hits: Option<i64>,
    pub max_hits: Option<i64>,
    pub min_turns: Option<i64>,
    pub max_turns: Option<i64>,
    pub drain: i64,
    pub healing: i64,
    pub crit_rate: i64,
    pub ailment_chance: i64,
    pub flinch_chance: i64,
    pub stat_chance: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct MoveStatChange {
    pub change: i64,
    pub stat: NamedApiResource<Stat>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PastMoveStatValues {
    pub accuracy: Option<i64>,
    pub effect_chance: Option<i64>,
    pub power: Option<i64>,
    pub pp: Option<i64>,
    pub effect_entries: Vec<VerboseEffect>,
    #[serde(rename = "type")]
    pub type_: Option<NamedApiResource<Type>>,
    pub version_group: NamedApiResource<VersionGroup>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct MoveAilment {
    pub id: i64,
    pub name: String,
    pub moves: Vec<NamedApiResource<Move>>,
    pub names: Vec<Name>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct MoveBattleStyle {
    pub id: i64,
    pub name: String,
    pub names: Vec<Name>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct MoveCategory {
    pub id: i64,
    pub name: String,
    pub moves: Vec<NamedApiResource<Move>>,
    pub descriptions: Vec<Description>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct MoveDamageClass {
    pub id: i64,
    pub name: String,
    pub descriptions: Vec<Description>,
    pub moves: Vec<NamedApiResource<Move>>,
    pub names: Vec<Name>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct MoveLearnMethod {
    pub id: i64,
    pub name: String,
    pub descriptions: Vec<Description>,
    pub names: Vec<Name>,
    pub version_groups: Vec<NamedApiResource<VersionGroup>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct MoveTarget {
    pub id: i64,
    pub name: String,
    pub descriptions: Vec<Description>,
    pub moves: Vec<NamedApiResource<Move>>,
    pub names: Vec<Name>,
}
