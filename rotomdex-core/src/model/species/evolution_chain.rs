use std::task::Poll;

use rustemon::{
    Follow,
    model::{evolution::EvolutionChain, resource::ApiResource},
};
use tracing::info_span;

use crate::model::Fetchable;

#[derive(Debug)]
pub(crate) struct ModelEvolutionChain {
    chain: EvolutionChain,
}

impl Fetchable for ModelEvolutionChain {
    type Request = ApiResource<EvolutionChain>;
    async fn fetch(request: Self::Request, ctx: crate::context::ModelContext) -> color_eyre::eyre::Result<Self> {
        let chain = request.follow(&ctx.pkmn_client).await?;
        Ok(Self { chain })
    }
    fn poll(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<()> {
        Poll::Pending
    }
    fn is_loaded(&self) -> bool {
        true
    }
    fn fetch_span(_request: &Self::Request) -> tracing::Span {
        info_span!("fetch_evolution_chain")
    }
}

pub(crate) struct ModelChainLink {}
