use color_eyre::eyre::{Error, eyre};
use rustemon::model::pokemon::{Pokemon, PokemonType};

#[derive(Clone, Debug)]
pub struct OfflineVariant {
    pub pkmn: Pokemon,
    pub types: Types,
}

impl TryFrom<Pokemon> for OfflineVariant {
    type Error = Error;
    fn try_from(value: Pokemon) -> std::prelude::v1::Result<Self, Self::Error> {
        let types = Types::try_from(&value.types[..])?;
        Ok(Self { pkmn: value, types })
    }
}

#[derive(Clone, Debug)]
pub struct Types {
    primary: Type,
    secondary: Option<Type>,
}

impl TryFrom<&[PokemonType]> for Types {
    type Error = Error;
    fn try_from(value: &[PokemonType]) -> std::prelude::v1::Result<Self, Self::Error> {
        let primary = value
            .iter()
            .find(|model| model.slot == 1)
            .map(Type::try_from)
            .transpose()?
            .ok_or_else(|| eyre!("no primary type found"))?;
        let secondary = value
            .iter()
            .find(|model| model.slot == 2)
            .map(Type::try_from)
            .transpose()?;

        Ok(Self { primary, secondary })
    }
}

#[derive(Clone, Debug)]
pub enum Type {
    Normal,
    Fire,
    Water,
    Electric,
    Grass,
    Ice,
    Fighting,
    Poison,
    Ground,
    Flying,
    Psychic,
    Bug,
    Rock,
    Ghost,
    Dragon,
    Dark,
    Steel,
    Fairy,
}

impl TryFrom<&PokemonType> for Type {
    type Error = Error;
    fn try_from(value: &PokemonType) -> std::prelude::v1::Result<Self, Self::Error> {
        let type_ = match value.type_.name.as_str() {
            "normal" => Self::Normal,
            "fire" => Self::Fire,
            "water" => Self::Water,
            "electric" => Self::Electric,
            "grass" => Self::Grass,
            "ice" => Self::Ice,
            "fighting" => Self::Fighting,
            "poison" => Self::Poison,
            "ground" => Self::Ground,
            "flying" => Self::Flying,
            "psychic" => Self::Psychic,
            "bug" => Self::Bug,
            "rock" => Self::Rock,
            "ghost" => Self::Ghost,
            "dragon" => Self::Dragon,
            "dark" => Self::Dark,
            "steel" => Self::Steel,
            "fairy" => Self::Fairy,
            _ => return Err(eyre!("invalid type name found")),
        };
        Ok(type_)
    }
}
