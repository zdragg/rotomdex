use alloc::{string::String, vec::Vec};

use super::{
    games::VersionGroup,
    items::Item,
    locations::{Location, Region},
    moves::Move,
    pokemon::{Pokemon, PokemonSpecies, Type},
    resource::{Name, NamedApiResource},
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct EvolutionChain {
    pub id: i64,
    pub baby_trigger_item: Option<NamedApiResource<Item>>,
    pub chain: ChainLink,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct ChainLink {
    pub is_baby: bool,
    pub species: NamedApiResource<PokemonSpecies>,
    pub evolution_details: Vec<EvolutionDetail>,
    pub evolves_to: Vec<ChainLink>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct EvolutionDetail {
    pub version_group: NamedApiResource<VersionGroup>,
    pub is_default: bool,
    pub item: Option<NamedApiResource<Item>>,
    pub trigger: NamedApiResource<EvolutionTrigger>,
    pub gender: Option<i64>,
    pub held_item: Option<NamedApiResource<Item>>,
    pub known_move: Option<NamedApiResource<Move>>,
    pub known_move_type: Option<NamedApiResource<Type>>,
    pub location: Option<NamedApiResource<Location>>,
    pub min_level: Option<i64>,
    pub min_happiness: Option<i64>,
    pub min_beauty: Option<i64>,
    pub min_affection: Option<i64>,
    pub near_special_rock: bool,
    pub needs_multiplayer: bool,
    pub needs_overworld_rain: bool,
    pub party_species: Option<NamedApiResource<PokemonSpecies>>,
    pub party_type: Option<NamedApiResource<Type>>,
    pub relative_physical_stats: Option<i64>,
    pub time_of_day: String,
    pub trade_species: Option<NamedApiResource<PokemonSpecies>>,
    pub turn_upside_down: bool,
    pub region: Option<NamedApiResource<Region>>,
    pub required_pokemon_form: Option<NamedApiResource<Pokemon>>,
    pub evolved_pokemon_form: Option<NamedApiResource<Pokemon>>,
    pub used_move: Option<NamedApiResource<Move>>,
    pub min_move_count: Option<i64>,
    pub min_steps: Option<i64>,
    pub min_damage_taken: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct EvolutionTrigger {
    pub id: i64,
    pub name: String,
    pub names: Vec<Name>,
    pub pokemon_species: Vec<NamedApiResource<PokemonSpecies>>,
}
