use std::task::Poll;

use color_eyre::eyre::Result;
use rustemon::{
    Follow,
    model::{
        evolution::{ChainLink, EvolutionChain, EvolutionDetail},
        resource::ApiResource,
    },
};

use crate::{VersionGroup, context::ModelContext, model::Fetchable};

#[derive(Debug)]
pub(crate) struct ModelEvolutionChain {
    pub(crate) base: ModelChainLink,
}

impl Fetchable for ModelEvolutionChain {
    type Request = ApiResource<EvolutionChain>;
    async fn fetch(request: Self::Request, ctx: crate::context::ModelContext) -> Result<Self> {
        let chain = request.follow(&ctx.pkmn_client).await?;
        Ok(Self {
            base: ModelChainLink::new(chain.chain, &ctx)?,
        })
    }
    fn poll(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<()> {
        Poll::Pending
    }
    fn is_loaded(&self) -> bool {
        true
    }
    fn fetch_span(_request: &Self::Request) -> tracing::Span {
        tracing::info_span!("fetch_evolution")
    }
}

impl ModelEvolutionChain {
    pub(crate) fn get_views(&self) -> Vec<ModelChainLinkView<'_>> {
        self.base.get_view_with_child(0, true)
    }
}

#[derive(Debug)]
pub(crate) struct ModelChainLink {
    pub(crate) species_name: String,
    pub(crate) evolution_detail: ModelEvolutionDetail,
    pub(crate) evolves_to: Vec<ModelChainLink>,
}

impl ModelChainLink {
    fn new(link: ChainLink, ctx: &ModelContext) -> Result<Self> {
        Ok(Self {
            species_name: link.species.name,
            evolution_detail: ModelEvolutionDetail::new(link.evolution_details, ctx)?,
            evolves_to: link
                .evolves_to
                .into_iter()
                .map(|link| ModelChainLink::new(link, &ctx))
                .collect::<Result<Vec<_>>>()?,
        })
    }
    fn get_view_with_child(&self, depth: usize, last_in_depth: bool) -> Vec<ModelChainLinkView<'_>> {
        let view = ModelChainLinkView {
            species_name: &self.species_name,
            evolution_detail: &self.evolution_detail,
            depth,
            last_in_depth,
        };
        let mut vec = vec![view];
        vec.extend(
            self.evolves_to
                .iter()
                .enumerate()
                .map(|(i, link)| {
                    let last_in_depth = i + 1 == self.evolves_to.len();
                    link.get_view_with_child(depth + 1, last_in_depth)
                })
                .flatten(),
        );
        vec
    }
}

pub(crate) struct ModelChainLinkView<'a> {
    pub(crate) species_name: &'a str,
    pub(crate) evolution_detail: &'a ModelEvolutionDetail,
    pub(crate) depth: usize,
    pub(crate) last_in_depth: bool,
}

#[derive(Debug)]
pub(crate) struct ModelEvolutionDetail {
    // This part of the evolution chain does not exist if this is None
    pub(crate) inner: Option<EvolutionDetail>,
}

impl ModelEvolutionDetail {
    fn new(detail: Vec<EvolutionDetail>, ctx: &ModelContext) -> Result<Self> {
        let detail = detail
            .into_iter()
            .filter(|detail| {
                let maybe_version_group = detail.version_group.name.parse::<VersionGroup>().ok();
                matches!(maybe_version_group, Some(version_group) if version_group == ctx.version.version_group())
            })
            .next();

        Ok(Self { inner: detail })
    }
}
