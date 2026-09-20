use alloc::{string::String, vec::Vec};

use super::{
    encounters::EncounterMethod,
    games::{Generation, Pokedex, Version, VersionGroup},
    pokemon::{Pokemon, PokemonSpecies},
    resource::{GenerationGameIndex, Name, NamedApiResource, VersionEncounterDetail},
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Location {
    pub id: i64,
    pub name: String,
    pub region: Option<NamedApiResource<Region>>,
    pub names: Vec<Name>,
    pub game_indices: Vec<GenerationGameIndex>,
    pub areas: Vec<NamedApiResource<LocationArea>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct LocationArea {
    pub id: i64,
    pub name: String,
    pub game_index: i64,
    pub encounter_method_rates: Vec<EncounterMethodRate>,
    pub location: NamedApiResource<Location>,
    pub names: Vec<Name>,
    pub pokemon_encounters: Vec<PokemonEncounter>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct EncounterMethodRate {
    pub encounter_method: NamedApiResource<EncounterMethod>,
    pub version_details: Vec<EncounterVersionDetails>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct EncounterVersionDetails {
    pub rate: i64,
    pub version: NamedApiResource<Version>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PokemonEncounter {
    pub pokemon: NamedApiResource<Pokemon>,
    pub version_details: Vec<VersionEncounterDetail>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PalParkArea {
    pub id: i64,
    pub name: String,
    pub names: Vec<Name>,
    pub pokemon_encounters: Vec<PalParkEncounterSpecies>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PalParkEncounterSpecies {
    pub base_score: i64,
    pub rate: i64,
    pub pokemon_species: NamedApiResource<PokemonSpecies>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Region {
    pub id: i64,
    pub locations: Vec<NamedApiResource<Location>>,
    pub name: String,
    pub names: Vec<Name>,
    pub main_generation: Option<NamedApiResource<Generation>>,
    pub pokedexes: Vec<NamedApiResource<Pokedex>>,
    pub version_groups: Vec<NamedApiResource<VersionGroup>>,
}
