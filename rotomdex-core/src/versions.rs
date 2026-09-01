use strum::{Display, EnumCount, EnumString, VariantArray};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display, EnumString, Default, EnumCount, VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Version {
    Red,
    Blue,
    Yellow,
    Gold,
    Silver,
    Crystal,
    Ruby,
    Sapphire,
    Emerald,
    #[strum(serialize = "firered")]
    FireRed,
    #[strum(serialize = "leafgreen")]
    LeafGreen,
    Diamond,
    Pearl,
    Platinum,
    #[strum(serialize = "heartgold")]
    HeartGold,
    #[strum(serialize = "soulsilver")]
    SoulSilver,
    Black,
    White,
    #[strum(serialize = "black-2")]
    Black2,
    #[strum(serialize = "white-2")]
    White2,
    X,
    Y,
    OmegaRuby,
    AlphaSapphire,
    Sun,
    Moon,
    UltraSun,
    UltraMoon,
    LetsGoPikachu,
    LetsGoEevee,
    #[default]
    Sword,
    Shield,
    BrilliantDiamond,
    ShiningPearl,
    LegendsArceus,
    Scarlet,
    Violet,
    LegendsZa,
}

impl Version {
    pub(crate) const fn abbreviation(&self) -> &'static str {
        match self {
            Self::Red => "R",
            Self::Blue => "B",
            Self::Yellow => "Y",
            Self::Gold => "G",
            Self::Silver => "S",
            Self::Crystal => "C",
            Self::Ruby => "R",
            Self::Sapphire => "S",
            Self::Emerald => "E",
            Self::FireRed => "FR",
            Self::LeafGreen => "LG",
            Self::Diamond => "D",
            Self::Pearl => "P",
            Self::Platinum => "P",
            Self::HeartGold => "HG",
            Self::SoulSilver => "SS",
            Self::Black => "B",
            Self::White => "W",
            Self::Black2 => "B2",
            Self::White2 => "W2",
            Self::X => "X",
            Self::Y => "Y",
            Self::OmegaRuby => "OR",
            Self::AlphaSapphire => "AS",
            Self::Sun => "S",
            Self::Moon => "M",
            Self::UltraSun => "US",
            Self::UltraMoon => "UM",
            Self::LetsGoPikachu => "LGP",
            Self::LetsGoEevee => "LGE",
            Self::Sword => "S",
            Self::Shield => "S",
            Self::BrilliantDiamond => "BD",
            Self::ShiningPearl => "SP",
            Self::LegendsArceus => "PLA",
            Self::Scarlet => "S",
            Self::Violet => "V",
            Self::LegendsZa => "PLZA",
        }
    }

    // Inspired by https://bulbapedia.bulbagarden.net/wiki/Help%3AColor_templates#Core%20series%20games
    pub(crate) const fn color(&self) -> &'static str {
        match self {
            Self::Red => "#DA3914",
            Self::Blue => "#2E50D8",
            Self::Yellow => "#FFD733",
            Self::Gold => "#DAA520",
            Self::Silver => "#C0C0C0",
            Self::Crystal => "#4FD9FF",
            Self::Ruby => "#CD2236",
            Self::Sapphire => "#3D51A7",
            Self::Emerald => "#009652",
            Self::FireRed => "#F15C01",
            Self::LeafGreen => "#9FDC00",
            Self::Diamond => "#90BEED",
            Self::Pearl => "#DD7CB1",
            Self::Platinum => "#A0A08D",
            Self::HeartGold => "#E8B502",
            Self::SoulSilver => "#AAB9CF",
            Self::Black => "#444444",
            Self::White => "#E1E1E1",
            Self::Black2 => "#303E51",
            Self::White2 => "#EBC5C3",
            Self::X => "#025DA6",
            Self::Y => "#EA1A3E",
            Self::OmegaRuby => "#AB2813",
            Self::AlphaSapphire => "#26649C",
            Self::Sun => "#F1912B",
            Self::Moon => "#5599CA",
            Self::UltraSun => "#E95B2B",
            Self::UltraMoon => "#226DB5",
            Self::LetsGoPikachu => "#F5DA26",
            Self::LetsGoEevee => "#D4924B",
            Self::Sword => "#00A1E9",
            Self::Shield => "#BF004F",
            Self::BrilliantDiamond => "#44BAE5",
            Self::ShiningPearl => "#DA7D99",
            Self::LegendsArceus => "#36597B",
            Self::Scarlet => "#F34134",
            Self::Violet => "#8334B7",
            Self::LegendsZa => "#31CA56",
        }
    }

    pub(crate) fn version_group(&self) -> VersionGroup {
        match self {
            Self::Red | Self::Blue => VersionGroup::RedBlue,
            Self::Yellow => VersionGroup::Yellow,
            Self::Gold | Self::Silver => VersionGroup::GoldSilver,
            Self::Crystal => VersionGroup::Crystal,
            Self::Ruby | Self::Sapphire => VersionGroup::RubySapphire,
            Self::Emerald => VersionGroup::Emerald,
            Self::FireRed | Self::LeafGreen => VersionGroup::FireRedLeafGreen,
            Self::Diamond | Self::Pearl => VersionGroup::DiamondPearl,
            Self::Platinum => VersionGroup::Platinum,
            Self::HeartGold | Self::SoulSilver => VersionGroup::HeartGoldSoulSilver,
            Self::Black | Self::White => VersionGroup::BlackWhite,
            Self::Black2 | Self::White2 => VersionGroup::Black2White2,
            Self::X | Self::Y => VersionGroup::XY,
            Self::OmegaRuby | Self::AlphaSapphire => VersionGroup::OmegaRubyAlphaSapphire,
            Self::Sun | Self::Moon => VersionGroup::SunMoon,
            Self::UltraSun | Self::UltraMoon => VersionGroup::UltraSunUltraMoon,
            Self::LetsGoPikachu | Self::LetsGoEevee => VersionGroup::LetsGoPikachuLetsGoEevee,
            Self::Sword | Self::Shield => VersionGroup::SwordShield,
            Self::BrilliantDiamond | Self::ShiningPearl => VersionGroup::BrilliantDiamondShiningPearl,
            Self::LegendsArceus => VersionGroup::LegendsArceus,
            Self::Scarlet | Self::Violet => VersionGroup::ScarletViolet,
            Self::LegendsZa => VersionGroup::LegendsZa,
        }
    }
    pub(crate) fn generation(&self) -> Generation {
        self.version_group().generation()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, EnumCount, VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum VersionGroup {
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
    ScarletViolet,
    LegendsZa,
}

impl VersionGroup {
    pub(crate) fn versions(&self) -> Vec<Version> {
        match self {
            Self::RedBlue => vec![Version::Red, Version::Blue],
            Self::Yellow => vec![Version::Yellow],
            Self::GoldSilver => vec![Version::Gold, Version::Silver],
            Self::Crystal => vec![Version::Crystal],
            Self::RubySapphire => vec![Version::Ruby, Version::Sapphire],
            Self::Emerald => vec![Version::Emerald],
            Self::FireRedLeafGreen => vec![Version::FireRed, Version::LeafGreen],
            Self::DiamondPearl => vec![Version::Diamond, Version::Pearl],
            Self::Platinum => vec![Version::Platinum],
            Self::HeartGoldSoulSilver => vec![Version::HeartGold, Version::SoulSilver],
            Self::BlackWhite => vec![Version::Black, Version::White],
            Self::Black2White2 => vec![Version::Black2, Version::White2],
            Self::XY => vec![Version::X, Version::Y],
            Self::OmegaRubyAlphaSapphire => vec![Version::OmegaRuby, Version::AlphaSapphire],
            Self::SunMoon => vec![Version::Sun, Version::Moon],
            Self::UltraSunUltraMoon => vec![Version::UltraSun, Version::UltraMoon],
            Self::LetsGoPikachuLetsGoEevee => vec![Version::LetsGoPikachu, Version::LetsGoEevee],
            Self::SwordShield => vec![Version::Sword, Version::Shield],
            Self::BrilliantDiamondShiningPearl => vec![Version::BrilliantDiamond, Version::ShiningPearl],
            Self::LegendsArceus => vec![Version::LegendsArceus],
            Self::ScarletViolet => vec![Version::Scarlet, Version::Violet],
            Self::LegendsZa => vec![Version::LegendsZa],
        }
    }

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
