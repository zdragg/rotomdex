use color_eyre::eyre::{Result, eyre};
use colorgrad::{Gradient, GradientBuilder, LinearGradient};
use ratatui::text::Span;
use rustemon::model::pokemon::{PokemonType, PokemonTypePast};

use crate::{Generation, ModelContext};

#[derive(Clone, Debug)]
pub(crate) struct ModelTypes {
    pub(crate) primary: ModelType,
    pub(crate) secondary: Option<ModelType>,
}

impl ModelTypes {
    pub(crate) fn new(current: &[PokemonType], past: &[PokemonTypePast], ctx: ModelContext) -> Result<Self> {
        let target_generation = ctx.settings.version.generation();
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
            .map(ModelType::new)
            .transpose()?
            .ok_or_else(|| eyre!("no primary type found"))?;
        let secondary = relevant_types
            .iter()
            .find(|model| model.slot == 2)
            .map(ModelType::new)
            .transpose()?;

        Ok(Self { primary, secondary })
    }

    pub(crate) fn spans_iter(&self, name: &str) -> Vec<Span<'_>> {
        let mut builder = GradientBuilder::new();
        let grad = if let Some(secondary) = &self.secondary {
            builder.html_colors(&[self.primary.color(), secondary.color()])
        } else {
            builder.html_colors(&[self.primary.color()])
        }
        .mode(colorgrad::BlendMode::Oklab)
        .build::<LinearGradient>()
        .unwrap();

        let char_count = name.chars().count();
        grad.colors_iter(char_count)
            .zip(name.chars())
            .map(|(color, ch)| {
                let color = color.clamp();
                let tui_color = ratatui::style::Color::Rgb(
                    (color.r * 255.0).round() as u8,
                    (color.g * 255.0).round() as u8,
                    (color.b * 255.0).round() as u8,
                );
                Span::styled(ch.to_string(), tui_color)
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
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
    pub(crate) fn new(value: &PokemonType) -> Result<Self> {
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

impl ModelType {
    pub(crate) const fn color(&self) -> &'static str {
        // Source: https://bulbapedia.bulbagarden.net/wiki/Help%3AColor_templates#Video_game_types
        match self {
            Self::Normal => "#9FA19F",
            Self::Fire => "#E62829",
            Self::Water => "#2980EF",
            Self::Electric => "#FAC000",
            Self::Grass => "#3FA129",
            Self::Ice => "#3DCEF3",
            Self::Fighting => "#FF8000",
            Self::Poison => "#9141CB",
            Self::Ground => "#915121",
            Self::Flying => "#81B9EF",
            Self::Psychic => "#EF4179",
            Self::Bug => "#91A119",
            Self::Rock => "#AFA981",
            Self::Ghost => "#704170",
            Self::Dragon => "#5060E1",
            Self::Dark => "#624D4E",
            Self::Steel => "#60A1B8",
            Self::Fairy => "#EF70EF",
        }
    }
}
