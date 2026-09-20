use alloc::{string::String, vec::Vec};

use super::{
    berries::BerryFlavor,
    moves::Move,
    resource::{Effect, FlavorText, NamedApiResource},
    utility::Language,
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct ContestType {
    pub id: i64,
    pub name: String,
    pub berry_flavor: NamedApiResource<BerryFlavor>,
    pub names: Vec<ContestName>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct ContestName {
    pub name: Option<String>,
    pub color: Option<String>,
    pub language: Option<NamedApiResource<Language>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct ContestEffect {
    pub id: i64,
    pub appeal: i64,
    pub jam: i64,
    pub effect_entries: Vec<Effect>,
    pub flavor_text_entries: Vec<FlavorText>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct SuperContestEffect {
    pub id: i64,
    pub appeal: i64,
    pub flavor_text_entries: Vec<FlavorText>,
    pub moves: Vec<NamedApiResource<Move>>,
}
