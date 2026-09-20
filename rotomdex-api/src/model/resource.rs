use alloc::{string::String, vec::Vec};

use core::marker::PhantomData;

use super::{
    encounters::{EncounterConditionValue, EncounterMethod},
    games::{Generation, Version, VersionGroup},
    machines::Machine,
    utility::Language,
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct NamedApiResource<T> {
    pub name: String,
    pub url: String,
    #[serde(skip)]
    _marker: PhantomData<T>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct ApiResource<T> {
    pub url: String,
    #[serde(skip)]
    _marker: PhantomData<T>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Description {
    pub description: String,
    pub language: NamedApiResource<Language>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Effect {
    pub effect: String,
    pub language: NamedApiResource<Language>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Encounter {
    pub min_level: i64,
    pub max_level: i64,
    pub condition_values: Vec<NamedApiResource<EncounterConditionValue>>,
    pub chance: i64,
    pub method: NamedApiResource<EncounterMethod>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct FlavorText {
    pub flavor_text: String,
    pub language: NamedApiResource<Language>,
    pub version: Option<NamedApiResource<Version>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct GenerationGameIndex {
    pub game_index: i64,
    pub generation: NamedApiResource<Generation>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct MachineVersionDetail {
    pub machine: ApiResource<Machine>,
    pub version_group: NamedApiResource<VersionGroup>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Name {
    pub name: String,
    pub language: NamedApiResource<Language>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct VerboseEffect {
    pub effect: String,
    pub short_effect: String,
    pub language: NamedApiResource<Language>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct VersionEncounterDetail {
    pub version: NamedApiResource<Version>,
    pub max_chance: i64,
    pub encounter_details: Vec<Encounter>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct VersionGameIndex {
    pub game_index: i64,
    pub version: NamedApiResource<Version>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct VersionGroupFlavorText {
    pub text: String,
    pub language: NamedApiResource<Language>,
    pub version_group: NamedApiResource<VersionGroup>,
}
