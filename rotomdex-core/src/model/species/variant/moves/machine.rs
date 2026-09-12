use std::fmt::Display;

use color_eyre::eyre::eyre;
use rustemon::{
    Follow,
    model::{machines::Machine, resource::ApiResource},
};

use crate::model::Fetchable;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub(crate) struct ModelMachine {
    pub(crate) type_: ModelMachineType,
    pub(crate) id: u32,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ModelMachineType {
    Hidden,
    Machine,
    Record,
}

impl Display for ModelMachineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Machine => write!(f, "tm"),
            Self::Record => write!(f, "tr"),
            Self::Hidden => write!(f, "hm"),
        }
    }
}

impl Fetchable for ModelMachine {
    type Request = ApiResource<Machine>;
    async fn fetch(request: Self::Request, ctx: crate::context::ModelContext) -> color_eyre::eyre::Result<Self> {
        let machine = request.follow(&ctx.pkmn_client).await?;
        let (type_, id) = if let Some(rest) = machine.item.name.strip_prefix("tm") {
            (ModelMachineType::Machine, rest.parse::<u32>()?)
        } else if let Some(rest) = machine.item.name.strip_prefix("tr") {
            (ModelMachineType::Record, rest.parse::<u32>()?)
        } else if let Some(rest) = machine.item.name.strip_prefix("hm") {
            (ModelMachineType::Hidden, rest.parse::<u32>()?)
        } else {
            return Err(eyre!("Machine id cannot be parsed"));
        };

        Ok(Self { type_, id })
    }
    fn poll(&mut self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<()> {
        std::task::Poll::Pending
    }
    fn fetch_span(_request: &Self::Request) -> tracing::Span {
        tracing::info_span!("fetch_machine")
    }
}

impl Display for ModelMachine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.type_, self.id)
    }
}
