use core::{
    fmt::{self, Display},
    task::{Context, Poll},
};

use color_eyre::eyre::eyre;
use rotomdex_api::{
    Follow,
    client::Client,
    model::{machines::Machine, resource::ApiResource},
};

use crate::{Settings, data::resource::Fetchable};

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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Machine => write!(f, "tm"),
            Self::Record => write!(f, "tr"),
            Self::Hidden => write!(f, "hm"),
        }
    }
}

impl Fetchable for ModelMachine {
    type Request = ApiResource<Machine>;
    async fn fetch(
        request: Self::Request,
        client: Client,
        _settings: Settings,
    ) -> color_eyre::eyre::Result<Self> {
        let machine = request.follow(&client).await?;
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
    fn poll(&mut self, _cx: &mut Context<'_>) -> Poll<()> {
        Poll::Pending
    }
    fn fetch_span(_request: &Self::Request) -> tracing::Span {
        tracing::info_span!("fetch_machine")
    }
}

impl Display for ModelMachine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.type_, self.id)
    }
}
