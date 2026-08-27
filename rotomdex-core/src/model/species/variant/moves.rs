use std::{collections::HashMap, task::Poll};

use color_eyre::eyre::Result;
use rustemon::{
    Follow,
    model::{
        moves::Move,
        pokemon::{PokemonMove, PokemonMoveVersion},
        resource::NamedApiResource,
    },
};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};

use crate::{
    FetchContext,
    model::{Fetchable, Resource},
};

#[derive(Debug)]
pub(crate) struct ModelMoves {
    map: HashMap<ModelVersionGroup, Vec<ModelVersionMove>>,
}

impl ModelMoves {
    pub(crate) fn new(moves: &[PokemonMove], ctx: FetchContext) -> Result<Self> {
        let mut map: HashMap<_, Vec<_>> = HashMap::new();
        for m in moves {
            for move_version in &m.version_group_details {
                let Ok(version) = move_version.version_group.name.parse() else {
                    continue;
                };
                let Some(method) = ModelMoveLearnMethod::from(move_version.clone()) else {
                    continue;
                };
                let move_resource = Resource::<ModelMove>::fetch(m.move_.clone(), ctx.clone(), true);
                map.entry(version).or_default().push(ModelVersionMove {
                    name: m.move_.name.clone(),
                    learn_method: method,
                    resource: move_resource,
                });
            }
        }
        Ok(Self { map })
    }

    pub(crate) fn poll(&mut self, cx: &mut std::task::Context<'_>) -> Poll<()> {
        if self
            .map
            .values_mut()
            .flatten()
            .fold(false, |is_ready, move_| is_ready | move_.resource.poll(cx).is_ready())
        {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }

    pub(crate) fn versions_cnt(&self) -> usize {
        self.map.len()
    }

    pub(crate) fn get_versions(&self) -> Vec<ModelVersionGroup> {
        ModelVersionGroup::iter()
            .filter(|version| self.map.contains_key(version))
            .collect()
    }

    pub(crate) fn iter_moves(&self, move_gen_idx: usize) -> Option<&[ModelVersionMove]> {
        let version_group = ModelVersionGroup::iter()
            .filter(|version| self.map.contains_key(version))
            .nth(move_gen_idx)?;
        self.map.get(&version_group).map(|vec| &vec[..])
    }
}

#[derive(Debug)]
pub(crate) struct ModelVersionMove {
    pub(crate) name: String,
    pub(crate) learn_method: ModelMoveLearnMethod,
    pub(crate) resource: Resource<ModelMove>,
}

impl ModelVersionMove {
    pub(crate) fn undefer(&self) {
        self.resource.undefer();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumIter, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum ModelVersionGroup {
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

impl ModelVersionGroup {
    pub(crate) fn abbreviation(&self) -> &'static str {
        match &self {
            Self::RedBlue => "rb",
            Self::Yellow => "y",
            Self::GoldSilver => "gs",
            Self::Crystal => "c",
            Self::RubySapphire => "rs",
            Self::Emerald => "e",
            Self::FireRedLeafGreen => "frlg",
            Self::DiamondPearl => "dp",
            Self::Platinum => "pt",
            Self::HeartGoldSoulSilver => "hgss",
            Self::BlackWhite => "bw",
            Self::Black2White2 => "b2w2",
            Self::XY => "xy",
            Self::OmegaRubyAlphaSapphire => "oras",
            Self::SunMoon => "sm",
            Self::UltraSunUltraMoon => "usum",
            Self::LetsGoPikachuLetsGoEevee => "lgpe",
            Self::SwordShield => "swsh",
            Self::BrilliantDiamondShiningPearl => "bdsp",
            Self::LegendsArceus => "pla",
            Self::ScarletViolet => "sv",
            Self::LegendsZa => "plza",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ModelMoveLearnMethod {
    LevelUp(u32),
    Egg,
    Tutor,
    Machine,
    // TODO: implement the rest under https://pokeapi.co/api/v2/move-learn-method/{id}
}

impl ModelMoveLearnMethod {
    fn from(value: PokemonMoveVersion) -> Option<Self> {
        let method = match value.move_learn_method.name.as_str() {
            "level-up" => Self::LevelUp(value.level_learned_at as u32),
            "egg" => Self::Egg,
            "tutor" => Self::Tutor,
            "machine" => Self::Machine,
            _ => {
                tracing::warn!("unsupported move learn method {}", value.move_learn_method.name);
                return None;
            }
        };
        Some(method)
    }
}

#[derive(Debug)]
pub(crate) struct ModelMove {
    inner: Move,
}

impl Fetchable for ModelMove {
    type Request = NamedApiResource<Move>;
    async fn fetch(request: Self::Request, ctx: crate::FetchContext) -> color_eyre::eyre::Result<Self> {
        let move_ = request.follow(&ctx.pkmn_client).await?;
        Ok(Self { inner: move_ })
    }
    fn is_loaded(&self) -> bool {
        true
    }
    fn poll(&mut self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<()> {
        Poll::Pending
    }
    fn fetch_span(request: &Self::Request) -> tracing::Span {
        tracing::info_span!("fetch_move", move_ = %request.name)
    }
}
