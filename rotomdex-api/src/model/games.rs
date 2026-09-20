use alloc::{string::String, vec::Vec};

use super::{
    locations::Region,
    moves::{Move, MoveLearnMethod},
    pokemon::{Ability, PokemonSpecies, Type},
    resource::{Description, Name, NamedApiResource},
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Generation {
    pub id: i64,
    pub name: String,
    pub abilities: Vec<NamedApiResource<Ability>>,
    pub names: Vec<Name>,
    pub main_region: NamedApiResource<Region>,
    pub moves: Vec<NamedApiResource<Move>>,
    pub pokemon_species: Vec<NamedApiResource<PokemonSpecies>>,
    pub types: Vec<NamedApiResource<Type>>,
    pub version_groups: Vec<NamedApiResource<VersionGroup>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Pokedex {
    pub id: i64,
    pub name: String,
    pub is_main_series: bool,
    pub descriptions: Vec<Description>,
    pub names: Vec<Name>,
    pub pokemon_entries: Vec<PokemonEntry>,
    pub region: Option<NamedApiResource<Region>>,
    pub version_groups: Vec<NamedApiResource<VersionGroup>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PokemonEntry {
    pub entry_number: i64,
    pub pokemon_species: NamedApiResource<PokemonSpecies>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Version {
    pub id: i64,
    pub name: String,
    pub names: Vec<Name>,
    pub version_group: NamedApiResource<VersionGroup>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct VersionGroup {
    pub id: i64,
    pub name: String,
    pub order: i64,
    pub generation: NamedApiResource<Generation>,
    pub move_learn_methods: Vec<NamedApiResource<MoveLearnMethod>>,
    pub pokedexes: Vec<NamedApiResource<Pokedex>>,
    pub regions: Vec<NamedApiResource<Region>>,
    pub versions: Vec<NamedApiResource<Version>>,
}
