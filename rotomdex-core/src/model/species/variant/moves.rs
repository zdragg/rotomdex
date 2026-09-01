use std::task::Poll;

use color_eyre::eyre::Result;
use rustemon::{
    Follow,
    model::{
        moves::Move,
        pokemon::{PokemonMove, PokemonMoveVersion},
        resource::NamedApiResource,
    },
};
use strum::{Display, EnumCount, EnumIter};

use crate::{
    ModelContext, VersionGroup,
    model::{Fetchable, Resource},
};

#[derive(Debug)]
pub(crate) struct ModelMoves {
    moves: Vec<ModelVersionMove>,
}

impl ModelMoves {
    pub(crate) fn new(moves: &[PokemonMove], ctx: ModelContext) -> Result<Self> {
        let mut vec: Vec<_> = vec![];
        for m in moves {
            for move_version in &m.version_group_details {
                let Ok(version) = move_version.version_group.name.parse::<VersionGroup>() else {
                    continue;
                };

                if version != ctx.version.version_group() {
                    continue;
                }

                let Some(learn_method) = ModelMoveLearnMethod::from(move_version.clone()) else {
                    continue;
                };
                let resource = Resource::<ModelMove>::fetch(m.move_.clone(), ctx.clone(), true);
                vec.push(ModelVersionMove {
                    name: m.move_.name.clone(),
                    learn_method,
                    resource,
                });
            }
        }
        Ok(Self { moves: vec })
    }

    pub(crate) fn poll(&mut self, cx: &mut std::task::Context<'_>) -> Poll<()> {
        if self
            .moves
            .iter_mut()
            .fold(false, |is_ready, move_| is_ready | move_.resource.poll(cx).is_ready())
        {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }

    pub(crate) fn get(&self) -> &[ModelVersionMove] {
        &self.moves[..]
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

#[derive(Debug, Clone, Copy, PartialEq, Display, EnumIter, EnumCount)]
pub(crate) enum ModelMoveLearnMethod {
    LevelUp(u32),
    Egg,
    Tutor,
    Machine,
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
    async fn fetch(request: Self::Request, ctx: ModelContext) -> Result<Self> {
        let move_ = request.follow(&ctx.pkmn_client).await?;
        Ok(Self { inner: move_ })
    }
    fn is_loaded(&self) -> bool {
        true
    }
    fn poll(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<()> {
        Poll::Pending
    }
    fn fetch_span(request: &Self::Request) -> tracing::Span {
        tracing::info_span!("fetch_move", move_ = %request.name)
    }
}
