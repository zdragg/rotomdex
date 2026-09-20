use alloc::{string::String, vec::Vec};

use super::{
    evolution::EvolutionChain,
    games::Version,
    pokemon::Pokemon,
    resource::{
        ApiResource, Description, Effect, GenerationGameIndex, MachineVersionDetail, Name,
        NamedApiResource, VerboseEffect, VersionGroupFlavorText,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Item {
    pub id: i64,
    pub name: String,
    pub cost: i64,
    pub fling_power: Option<i64>,
    pub fling_effect: Option<NamedApiResource<ItemFlingEffect>>,
    pub attributes: Vec<NamedApiResource<ItemAttribute>>,
    pub category: NamedApiResource<ItemCategory>,
    pub effect_entries: Vec<VerboseEffect>,
    pub flavor_text_entries: Vec<VersionGroupFlavorText>,
    pub game_indices: Vec<GenerationGameIndex>,
    pub names: Vec<Name>,
    pub sprites: ItemSprites,
    pub held_by_pokemon: Vec<ItemHolderPokemon>,
    pub baby_trigger_for: Option<ApiResource<EvolutionChain>>,
    pub machines: Vec<MachineVersionDetail>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct ItemSprites {
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct ItemHolderPokemon {
    pub pokemon: NamedApiResource<Pokemon>,
    pub version_details: Vec<ItemHolderPokemonVersionDetail>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct ItemHolderPokemonVersionDetail {
    pub rarity: i64,
    pub version: NamedApiResource<Version>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct ItemAttribute {
    pub id: i64,
    pub name: String,
    pub items: Vec<NamedApiResource<Item>>,
    pub names: Vec<Name>,
    pub descriptions: Vec<Description>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct ItemCategory {
    pub id: i64,
    pub name: String,
    pub items: Vec<NamedApiResource<Item>>,
    pub names: Vec<Name>,
    pub pocket: NamedApiResource<ItemPocket>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct ItemFlingEffect {
    pub id: i64,
    pub name: String,
    pub effect_entries: Vec<Effect>,
    pub items: Vec<NamedApiResource<Item>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct ItemPocket {
    pub id: i64,
    pub name: String,
    pub categories: Vec<NamedApiResource<ItemCategory>>,
    pub names: Vec<Name>,
}
