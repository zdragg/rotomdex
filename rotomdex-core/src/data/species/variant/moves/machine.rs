use core::{
    fmt::{self, Display},
    num::ParseIntError,
    task::{Context, Poll},
};

use alloc::string::String;
use rotomdex_api::{
    Client, Follow,
    model::{machines::Machine, resource::ApiResource},
};
use snafu::{ResultExt, Snafu};

use crate::{
    Settings,
    data::resource::{ArbitraryResourceError, Fetchable, ResourceResult},
};

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub(crate) struct ModelMachine {
    pub(crate) type_: ModelMachineType,
    pub(crate) id: u32,
}

#[derive(Debug, Snafu)]
enum MachineError {
    #[snafu(display("{name} has an unknown machine type"))]
    InvalidMachineName { name: String },

    #[snafu(display("{name} has an invalid machine number"))]
    InvalidMachineNumber { source: ParseIntError, name: String },
}

impl ArbitraryResourceError for MachineError {}

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
    ) -> ResourceResult<Self> {
        let machine = request.follow(&client).await?;
        let name = machine.item.name;

        let (type_, number) = if let Some(number) = name.strip_prefix("tm") {
            (ModelMachineType::Machine, number)
        } else if let Some(number) = name.strip_prefix("tr") {
            (ModelMachineType::Record, number)
        } else if let Some(number) = name.strip_prefix("hm") {
            (ModelMachineType::Hidden, number)
        } else {
            InvalidMachineNameSnafu { name: &name }.fail()?
        };

        let id = number
            .parse::<u32>()
            .context(InvalidMachineNumberSnafu { name })?;

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
