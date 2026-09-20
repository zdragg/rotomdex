use alloc::{string::String, vec::Vec};

use super::{
    contests::ContestType,
    items::Item,
    pokemon::Type,
    resource::{Name, NamedApiResource},
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Berry {
    pub id: i64,
    pub name: String,
    pub growth_time: i64,
    pub max_harvest: i64,
    pub natural_gift_power: i64,
    pub size: i64,
    pub smoothness: i64,
    pub soil_dryness: i64,
    pub firmness: NamedApiResource<BerryFirmness>,
    pub flavors: Vec<BerryFlavorMap>,
    pub item: NamedApiResource<Item>,
    pub natural_gift_type: NamedApiResource<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct BerryFlavorMap {
    pub potency: i64,
    pub flavor: NamedApiResource<BerryFlavor>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct BerryFirmness {
    pub id: i64,
    pub name: String,
    pub berries: Vec<NamedApiResource<Berry>>,
    pub names: Vec<Name>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct BerryFlavor {
    pub id: i64,
    pub name: String,
    pub berries: Vec<FlavorBerryMap>,
    pub contest_type: NamedApiResource<ContestType>,
    pub names: Vec<Name>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct FlavorBerryMap {
    pub potency: i64,
    pub berry: NamedApiResource<Berry>,
}
