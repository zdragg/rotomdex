use crate::{Generation, ModelContext};
use color_eyre::eyre::{Result, eyre};
use colorgrad::{GradientBuilder, LinearGradient};
use rustemon::model::pokemon::{PokemonType, PokemonTypePast};
use strum::{Display, EnumString};

#[derive(Clone, Debug)]
pub(crate) struct ModelTypes {
    pub(crate) primary: ModelType,
    pub(crate) secondary: Option<ModelType>,
}

impl ModelTypes {
    pub(crate) fn new(current: &[PokemonType], past: &[PokemonTypePast], ctx: ModelContext) -> Result<Self> {
        let target_generation = ctx.version.generation();
        let relevant_types = past
            .iter()
            .filter_map(|past_types| {
                let final_gen = past_types.generation.name.parse::<Generation>().ok()?;
                Some((final_gen, past_types.types.as_slice()))
            })
            .filter(|(final_gen, _)| *final_gen >= target_generation)
            .min_by_key(|(final_gen, _)| *final_gen)
            .map(|(_, types)| types)
            .unwrap_or(current);

        let primary = relevant_types
            .iter()
            .find(|model| model.slot == 1)
            .map(|model| model.type_.name.parse())
            .transpose()?
            .ok_or_else(|| eyre!("no primary type found"))?;
        let secondary = relevant_types
            .iter()
            .find(|model| model.slot == 2)
            .map(|model| model.type_.name.parse())
            .transpose()?;

        Ok(Self { primary, secondary })
    }

    pub(crate) fn gradient(&self) -> LinearGradient {
        let mut builder = GradientBuilder::new();
        let [r1, g1, b1] = self.primary.color();
        if let Some(secondary) = &self.secondary {
            let [r2, g2, b2] = secondary.color();
            builder.colors(&[
                csscolorparser::Color::from_rgba8(r1, g1, b1, 0),
                csscolorparser::Color::from_rgba8(r2, g2, b2, 0),
            ])
        } else {
            builder.colors(&[csscolorparser::Color::from_rgba8(r1, g1, b1, 0)])
        }
        .mode(colorgrad::BlendMode::Oklab)
        .build::<LinearGradient>()
        .unwrap()
    }
}

#[derive(Clone, Debug, EnumString, Display)]
#[strum(ascii_case_insensitive)]
pub(crate) enum ModelType {
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

impl ModelType {
    pub(crate) const fn color(&self) -> [u8; 3] {
        // Source: https://bulbapedia.bulbagarden.net/wiki/Help%3AColor_templates#Video_game_types
        match self {
            Self::Normal => [159, 161, 159],
            Self::Fire => [230, 40, 41],
            Self::Water => [41, 128, 239],
            Self::Electric => [250, 192, 0],
            Self::Grass => [63, 161, 41],
            Self::Ice => [61, 206, 243],
            Self::Fighting => [255, 128, 0],
            Self::Poison => [145, 65, 203],
            Self::Ground => [145, 81, 33],
            Self::Flying => [129, 185, 239],
            Self::Psychic => [239, 65, 121],
            Self::Bug => [145, 161, 25],
            Self::Rock => [175, 169, 129],
            Self::Ghost => [112, 65, 112],
            Self::Dragon => [80, 96, 225],
            Self::Dark => [98, 77, 78],
            Self::Steel => [96, 161, 184],
            Self::Fairy => [239, 112, 239],
        }
    }

    pub(crate) fn tui_color(&self) -> ratatui::style::Color {
        let [r, g, b] = self.color();
        ratatui::style::Color::Rgb(r, g, b)
    }
}
