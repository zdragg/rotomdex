use derive_builder::Builder;
use strum::{Display, EnumIter, EnumString};

#[derive(Builder, Clone, Copy)]
pub struct Settings {
    #[builder(default)]
    pub(crate) version: Version,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumIter, EnumString, Default)]
#[strum(serialize_all = "kebab-case")]
pub enum Version {
    RedBlue,
    Yellow,
    GoldSilver,
    Crystal,
    RubySapphire,
    Emerald,
    #[strum(serialize = "firered-leafgreen")]
    FireRedLeafGreen,
    DiamondPearl,
    Platinum,
    #[strum(serialize = "heartgold-soulsilver")]
    HeartGoldSoulSilver,
    BlackWhite,
    #[strum(serialize = "black-2-white-2")]
    Black2White2,
    #[strum(serialize = "x-y")]
    XY,
    OmegaRubyAlphaSapphire,
    SunMoon,
    UltraSunUltraMoon,
    LetsGoPikachuLetsGoEevee,
    SwordShield,
    BrilliantDiamondShiningPearl,
    LegendsArceus,
    #[default]
    ScarletViolet,
    LegendsZa,
}

impl Version {
    pub(crate) fn generation(&self) -> Generation {
        match self {
            Self::RedBlue | Self::Yellow => Generation::Gen1,
            Self::GoldSilver | Self::Crystal => Generation::Gen2,
            Self::RubySapphire | Self::Emerald | Self::FireRedLeafGreen => Generation::Gen3,
            Self::DiamondPearl | Self::Platinum | Self::HeartGoldSoulSilver => Generation::Gen4,
            Self::BlackWhite | Self::Black2White2 => Generation::Gen5,
            Self::XY | Self::OmegaRubyAlphaSapphire => Generation::Gen6,
            Self::SunMoon | Self::UltraSunUltraMoon | Self::LetsGoPikachuLetsGoEevee => Generation::Gen7,
            Self::SwordShield | Self::BrilliantDiamondShiningPearl | Self::LegendsArceus => Generation::Gen8,
            Self::ScarletViolet | Self::LegendsZa => Generation::Gen9,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumString)]
pub enum Generation {
    #[strum(serialize = "generation-i")]
    Gen1,
    #[strum(serialize = "generation-ii")]
    Gen2,
    #[strum(serialize = "generation-iii")]
    Gen3,
    #[strum(serialize = "generation-iv")]
    Gen4,
    #[strum(serialize = "generation-v")]
    Gen5,
    #[strum(serialize = "generation-vi")]
    Gen6,
    #[strum(serialize = "generation-vii")]
    Gen7,
    #[strum(serialize = "generation-viii")]
    Gen8,
    #[strum(serialize = "generation-ix")]
    Gen9,
}
