use rustemon::{
    Follow,
    model::{machines::Machine, resource::ApiResource},
};

use crate::model::Fetchable;

#[derive(Debug)]
pub(crate) struct ModelMachine {
    pub(crate) name: String,
}

impl Fetchable for ModelMachine {
    type Request = ApiResource<Machine>;
    async fn fetch(request: Self::Request, ctx: crate::context::ModelContext) -> color_eyre::eyre::Result<Self> {
        let machine = request.follow(&ctx.pkmn_client).await?;
        Ok(Self {
            name: machine.item.name,
        })
    }
    fn is_loaded(&self) -> bool {
        true
    }
    fn poll(&mut self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<()> {
        std::task::Poll::Pending
    }
    fn fetch_span(_request: &Self::Request) -> tracing::Span {
        tracing::info_span!("fetch_machine")
    }
}
