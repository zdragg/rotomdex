pub mod stats;
pub mod types;

use color_eyre::eyre::Error;
use rustemon::model::pokemon::Pokemon;

use crate::offline::{stats::OfflineStats, variant::types::OfflineTypes};

#[derive(Clone, Debug)]
pub struct OfflineVariant {
    pub pkmn: Pokemon,
    pub types: OfflineTypes,
    pub stats: OfflineStats,
}

impl TryFrom<Pokemon> for OfflineVariant {
    type Error = Error;
    fn try_from(value: Pokemon) -> std::prelude::v1::Result<Self, Self::Error> {
        let types = OfflineTypes::try_from(&value.types[..])?;
        let stats = OfflineStats::try_from(&value.stats[..])?;
        Ok(Self {
            pkmn: value,
            types,
            stats,
        })
    }
}
