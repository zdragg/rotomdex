use color_eyre::eyre::Error;
use rustemon::model::pokemon::Pokemon;

use crate::offline::OfflineTypes;

#[derive(Clone, Debug)]
pub struct OfflineVariant {
    pub pkmn: Pokemon,
    pub types: OfflineTypes,
}

impl TryFrom<Pokemon> for OfflineVariant {
    type Error = Error;
    fn try_from(value: Pokemon) -> std::prelude::v1::Result<Self, Self::Error> {
        let types = OfflineTypes::try_from(&value.types[..])?;
        Ok(Self { pkmn: value, types })
    }
}
