use color_eyre::eyre::{Error, eyre};
use colorgrad::{Color, Gradient, GradientBuilder, LinearGradient};
use ratatui::text::Span;
use rustemon::model::pokemon::PokemonType;

#[derive(Clone, Debug)]
pub struct OfflineTypes {
    pub primary: OfflineType,
    pub secondary: Option<OfflineType>,
}

impl OfflineTypes {
    pub fn spans_iter(&self, name: &str) -> Vec<Span<'_>> {
        let mut builder = GradientBuilder::new();
        let grad = if let Some(secondary) = &self.secondary {
            builder.colors(&[self.primary.color(), secondary.color()])
        } else {
            builder.colors(&[self.primary.color()])
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
                Span::styled(ch.to_ascii_uppercase().to_string(), tui_color)
            })
            .collect()
    }
}

impl TryFrom<&[PokemonType]> for OfflineTypes {
    type Error = Error;
    fn try_from(value: &[PokemonType]) -> std::prelude::v1::Result<Self, Self::Error> {
        let primary = value
            .iter()
            .find(|model| model.slot == 1)
            .map(OfflineType::try_from)
            .transpose()?
            .ok_or_else(|| eyre!("no primary type found"))?;
        let secondary = value
            .iter()
            .find(|model| model.slot == 2)
            .map(OfflineType::try_from)
            .transpose()?;

        Ok(Self { primary, secondary })
    }
}

#[derive(Clone, Debug)]
pub enum OfflineType {
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

impl TryFrom<&PokemonType> for OfflineType {
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

impl OfflineType {
    pub fn color(&self) -> Color {
        let rgb = |r: u8, g: u8, b: u8| Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        };

        // Source: https://bulbapedia.bulbagarden.net/wiki/Help%3AColor_templates#Video_game_types
        match self {
            Self::Normal => rgb(0x9F, 0xA1, 0x9F),
            Self::Fire => rgb(0xE6, 0x28, 0x29),
            Self::Water => rgb(0x29, 0x80, 0xEF),
            Self::Electric => rgb(0xFA, 0xC0, 0x00),
            Self::Grass => rgb(0x3F, 0xA1, 0x29),
            Self::Ice => rgb(0x3D, 0xCE, 0xF3),
            Self::Fighting => rgb(0xFF, 0x80, 0x00),
            Self::Poison => rgb(0x91, 0x41, 0xCB),
            Self::Ground => rgb(0x91, 0x51, 0x21),
            Self::Flying => rgb(0x81, 0xB9, 0xEF),
            Self::Psychic => rgb(0xEF, 0x41, 0x79),
            Self::Bug => rgb(0x91, 0xA1, 0x19),
            Self::Rock => rgb(0xAF, 0xA9, 0x81),
            Self::Ghost => rgb(0x70, 0x41, 0x70),
            Self::Dragon => rgb(0x50, 0x60, 0xE1),
            Self::Dark => rgb(0x62, 0x4D, 0x4E),
            Self::Steel => rgb(0x60, 0xA1, 0xB8),
            Self::Fairy => rgb(0xEF, 0x70, 0xEF),
        }
    }
}
