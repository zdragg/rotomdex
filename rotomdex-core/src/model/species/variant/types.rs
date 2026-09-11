use std::borrow::Cow;

use crate::{Generation, ModelContext};
use color_eyre::eyre::{Result, eyre};
use colorgrad::{GradientBuilder, LinearGradient};
use ratatui::{style::Stylize, text::Span};
use rustemon::model::pokemon::{PokemonType, PokemonTypePast};
use strum::{Display, EnumCount, EnumIter, EnumString, IntoEnumIterator};

#[derive(Clone, Copy, Debug)]
pub(crate) struct ModelTypes {
    pub(crate) primary: ModelType,
    pub(crate) secondary: Option<ModelType>,
}

impl ModelTypes {
    pub(crate) fn new(current: Vec<PokemonType>, past: Vec<PokemonTypePast>, ctx: &ModelContext) -> Result<Self> {
        let target_generation = ctx.version.generation();

        let relevant_types = past
            .into_iter()
            .filter_map(|past_types| {
                let final_gen = past_types.generation.name.parse::<Generation>().ok()?;
                Some((final_gen, past_types.types))
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

    pub(crate) fn def_effectiveness(&self) -> ModelTypeEffectiveness {
        let Some(secd) = self.secondary else {
            let mono = self.primary;
            return ModelTypeEffectiveness::new(&[], mono.def_2x(), mono.def_halfx(), &[], mono.def_0x());
        };

        let prim = self.primary;

        // 1 -> x/4, 2 -> x/2, 4 -> neutral, 8 -> 2x, 16 -> 4x

        let mut counter = [4u32; ModelType::COUNT];

        prim.def_0x().into_iter().for_each(|t| counter[*t as usize] = 0);
        secd.def_0x().into_iter().for_each(|t| counter[*t as usize] = 0);

        prim.def_2x().into_iter().for_each(|t| counter[*t as usize] <<= 1);
        secd.def_2x().into_iter().for_each(|t| counter[*t as usize] <<= 1);

        prim.def_halfx().into_iter().for_each(|t| counter[*t as usize] >>= 1);
        secd.def_halfx().into_iter().for_each(|t| counter[*t as usize] >>= 1);

        let [mut four, mut two, mut half, mut quarter, mut zero] = std::array::from_fn(|_| vec![]);

        for (num, type_) in counter.iter().zip(ModelType::iter()) {
            match num {
                0 => zero.push(type_),
                1 => quarter.push(type_),
                2 => half.push(type_),
                4 => {}
                8 => two.push(type_),
                16 => four.push(type_),
                _ => unreachable!(),
            }
        }

        ModelTypeEffectiveness::new(four, two, half, quarter, zero)
    }
}

type TypeSlice = Cow<'static, [ModelType]>;

#[derive(Default)]
pub(crate) struct ModelTypeEffectiveness {
    pub(crate) four: TypeSlice,
    pub(crate) two: TypeSlice,
    pub(crate) half: TypeSlice,
    pub(crate) quarter: TypeSlice,
    pub(crate) zero: TypeSlice,
}

impl ModelTypeEffectiveness {
    fn new(
        four: impl Into<TypeSlice>,
        two: impl Into<TypeSlice>,
        half: impl Into<TypeSlice>,
        quarter: impl Into<TypeSlice>,
        zero: impl Into<TypeSlice>,
    ) -> Self {
        ModelTypeEffectiveness {
            four: four.into(),
            two: two.into(),
            half: half.into(),
            quarter: quarter.into(),
            zero: zero.into(),
        }
    }

    pub(crate) fn into_spans(self) -> Vec<Span<'static>> {
        let Self {
            four,
            two,
            half,
            quarter,
            zero,
        } = self;

        let mut spans = vec![];

        let mut push_spans = |symbol: &'static str, types: Cow<'static, [ModelType]>| {
            if !types.is_empty() {
                spans.push(Span::raw(symbol).bold());
                for type_ in types.iter() {
                    let span = Span::styled(type_.initial(), type_.tui_color());
                    spans.push(span);
                }
                spans.push(Span::raw(" "));
            }
        };

        push_spans("4", four);
        push_spans("2", two);
        push_spans("½", half);
        push_spans("¼", quarter);
        push_spans("0", zero);

        spans
    }
}

#[derive(Clone, Copy, Debug, EnumString, Display, PartialEq, PartialOrd, Eq, Ord, EnumCount, EnumIter)]
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

    pub(crate) const fn tui_color(&self) -> ratatui::style::Color {
        let [r, g, b] = self.color();
        ratatui::style::Color::Rgb(r, g, b)
    }

    pub(crate) const fn initial(&self) -> &'static str {
        match self {
            Self::Normal => "N",
            Self::Fire => "F",
            Self::Water => "W",
            Self::Electric => "E",
            Self::Grass => "G",
            Self::Ice => "I",
            Self::Fighting => "F",
            Self::Poison => "P",
            Self::Ground => "G",
            Self::Flying => "F",
            Self::Psychic => "P",
            Self::Bug => "B",
            Self::Rock => "R",
            Self::Ghost => "G",
            Self::Dragon => "D",
            Self::Dark => "D",
            Self::Steel => "S",
            Self::Fairy => "F",
        }
    }

    pub(crate) const fn atk_2x(&self) -> &'static [Self] {
        match self {
            Self::Normal => &[],
            Self::Fire => &[Self::Grass, Self::Ice, Self::Bug, Self::Steel],
            Self::Water => &[Self::Fire, Self::Ground, Self::Rock],
            Self::Electric => &[Self::Water, Self::Flying],
            Self::Grass => &[Self::Water, Self::Ground, Self::Rock],
            Self::Ice => &[Self::Grass, Self::Ground, Self::Flying, Self::Dragon],
            Self::Fighting => &[Self::Normal, Self::Ice, Self::Rock, Self::Dark, Self::Steel],
            Self::Poison => &[Self::Grass, Self::Fairy],
            Self::Ground => &[Self::Fire, Self::Electric, Self::Poison, Self::Rock, Self::Steel],
            Self::Flying => &[Self::Grass, Self::Fighting, Self::Bug],
            Self::Psychic => &[Self::Fighting, Self::Poison],
            Self::Bug => &[Self::Grass, Self::Psychic, Self::Dark],
            Self::Rock => &[Self::Fire, Self::Ice, Self::Flying, Self::Bug],
            Self::Ghost => &[Self::Psychic, Self::Ghost],
            Self::Dragon => &[Self::Dragon],
            Self::Dark => &[Self::Psychic, Self::Ghost],
            Self::Steel => &[Self::Ice, Self::Rock, Self::Fairy],
            Self::Fairy => &[Self::Fighting, Self::Dragon, Self::Dark],
        }
    }

    pub(crate) const fn atk_halfx(&self) -> &'static [Self] {
        match self {
            Self::Normal => &[Self::Rock, Self::Steel],
            Self::Fire => &[Self::Fire, Self::Water, Self::Rock, Self::Dragon],
            Self::Water => &[Self::Water, Self::Grass, Self::Dragon],
            Self::Electric => &[Self::Grass, Self::Electric, Self::Dragon],
            Self::Grass => &[
                Self::Fire,
                Self::Grass,
                Self::Poison,
                Self::Flying,
                Self::Bug,
                Self::Dragon,
                Self::Steel,
            ],
            Self::Ice => &[Self::Fire, Self::Water, Self::Ice, Self::Steel],
            Self::Fighting => &[Self::Poison, Self::Flying, Self::Psychic, Self::Bug, Self::Fairy],
            Self::Poison => &[Self::Poison, Self::Ground, Self::Rock, Self::Ghost],
            Self::Ground => &[Self::Grass, Self::Bug],
            Self::Flying => &[Self::Electric, Self::Rock, Self::Steel],
            Self::Psychic => &[Self::Psychic, Self::Steel],
            Self::Bug => &[
                Self::Fire,
                Self::Fighting,
                Self::Poison,
                Self::Flying,
                Self::Ghost,
                Self::Steel,
                Self::Fairy,
            ],
            Self::Rock => &[Self::Fighting, Self::Ground, Self::Steel],
            Self::Ghost => &[Self::Dark],
            Self::Dragon => &[Self::Steel],
            Self::Dark => &[Self::Fighting, Self::Dark, Self::Fairy],
            Self::Steel => &[Self::Fire, Self::Water, Self::Electric, Self::Steel],
            Self::Fairy => &[Self::Fire, Self::Poison, Self::Steel],
        }
    }

    pub(crate) const fn atk_0x(&self) -> &'static [Self] {
        match self {
            Self::Normal => &[Self::Ghost],
            Self::Fire => &[],
            Self::Water => &[],
            Self::Electric => &[Self::Ground],
            Self::Grass => &[],
            Self::Ice => &[],
            Self::Fighting => &[Self::Ghost],
            Self::Poison => &[Self::Steel],
            Self::Ground => &[Self::Flying],
            Self::Flying => &[],
            Self::Psychic => &[Self::Dark],
            Self::Bug => &[],
            Self::Rock => &[],
            Self::Ghost => &[Self::Normal],
            Self::Dragon => &[Self::Fairy],
            Self::Dark => &[],
            Self::Steel => &[],
            Self::Fairy => &[],
        }
    }

    pub(crate) fn atk_effectiveness(&self) -> ModelTypeEffectiveness {
        ModelTypeEffectiveness::new(&[], self.atk_2x(), self.atk_halfx(), &[], self.atk_0x())
    }

    pub(crate) const fn def_2x(&self) -> &'static [Self] {
        match self {
            Self::Normal => &[Self::Fighting],
            Self::Fire => &[Self::Water, Self::Ground, Self::Rock],
            Self::Water => &[Self::Electric, Self::Grass],
            Self::Electric => &[Self::Ground],
            Self::Grass => &[Self::Fire, Self::Ice, Self::Poison, Self::Flying, Self::Bug],
            Self::Ice => &[Self::Fire, Self::Fighting, Self::Rock, Self::Steel],
            Self::Fighting => &[Self::Flying, Self::Psychic, Self::Fairy],
            Self::Poison => &[Self::Ground, Self::Psychic],
            Self::Ground => &[Self::Water, Self::Grass, Self::Ice],
            Self::Flying => &[Self::Electric, Self::Ice, Self::Rock],
            Self::Psychic => &[Self::Bug, Self::Ghost, Self::Dark],
            Self::Bug => &[Self::Fire, Self::Flying, Self::Rock],
            Self::Rock => &[Self::Water, Self::Grass, Self::Fighting, Self::Ground, Self::Steel],
            Self::Ghost => &[Self::Ghost, Self::Dark],
            Self::Dragon => &[Self::Ice, Self::Dragon, Self::Fairy],
            Self::Dark => &[Self::Fighting, Self::Bug, Self::Fairy],
            Self::Steel => &[Self::Fire, Self::Fighting, Self::Ground],
            Self::Fairy => &[Self::Poison, Self::Steel],
        }
    }

    pub(crate) const fn def_halfx(&self) -> &'static [Self] {
        match self {
            Self::Normal => &[],
            Self::Fire => &[Self::Fire, Self::Grass, Self::Ice, Self::Bug, Self::Steel, Self::Fairy],
            Self::Water => &[Self::Fire, Self::Water, Self::Ice, Self::Steel],
            Self::Electric => &[Self::Electric, Self::Flying, Self::Steel],
            Self::Grass => &[Self::Water, Self::Electric, Self::Grass, Self::Ground],
            Self::Ice => &[Self::Ice],
            Self::Fighting => &[Self::Bug, Self::Rock, Self::Dark],
            Self::Poison => &[Self::Grass, Self::Fighting, Self::Poison, Self::Bug, Self::Fairy],
            Self::Ground => &[Self::Poison, Self::Rock],
            Self::Flying => &[Self::Grass, Self::Fighting, Self::Bug],
            Self::Psychic => &[Self::Fighting, Self::Psychic],
            Self::Bug => &[Self::Grass, Self::Fighting, Self::Ground],
            Self::Rock => &[Self::Normal, Self::Fire, Self::Poison, Self::Flying],
            Self::Ghost => &[Self::Poison, Self::Bug],
            Self::Dragon => &[Self::Fire, Self::Water, Self::Electric, Self::Grass],
            Self::Dark => &[Self::Ghost, Self::Dark],
            Self::Steel => &[
                Self::Normal,
                Self::Grass,
                Self::Ice,
                Self::Flying,
                Self::Psychic,
                Self::Bug,
                Self::Rock,
                Self::Dragon,
                Self::Steel,
                Self::Fairy,
            ],
            Self::Fairy => &[Self::Fighting, Self::Bug, Self::Dark],
        }
    }

    pub(crate) const fn def_0x(&self) -> &'static [Self] {
        match self {
            Self::Normal => &[Self::Ghost],
            Self::Fire => &[],
            Self::Water => &[],
            Self::Electric => &[],
            Self::Grass => &[],
            Self::Ice => &[],
            Self::Fighting => &[],
            Self::Poison => &[],
            Self::Ground => &[Self::Electric],
            Self::Flying => &[Self::Ground],
            Self::Psychic => &[],
            Self::Bug => &[],
            Self::Rock => &[],
            Self::Ghost => &[Self::Normal, Self::Fighting],
            Self::Dragon => &[],
            Self::Dark => &[Self::Psychic],
            Self::Steel => &[Self::Poison],
            Self::Fairy => &[Self::Dragon],
        }
    }
}
