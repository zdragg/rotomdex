use core::task::{Context, Poll};

use alloc::{string::String, vec::Vec};
use itertools::Itertools;
use rotomdex_api::{Client, model::resource::FlavorText};
use tracing::info_span;

use crate::{
    Version,
    data::resource::{Derivable, ResourceResult},
};

#[derive(Debug)]
pub(crate) struct ModelFlavorText {
    pub(crate) text: Option<String>,
}

impl Derivable for ModelFlavorText {
    type Request = Vec<FlavorText>;
    fn derive(
        request: Self::Request,
        _client: &Client,
        settings: crate::Settings,
    ) -> ResourceResult<Self> {
        let text = request.into_iter().find_map(|entry| {
            if entry.version?.name.parse::<Version>().ok()? != settings.version {
                return None;
            }

            if entry.language.name != "en" {
                return None;
            }

            let text = entry.flavor_text.split_whitespace().join(" ");

            Some(text)
        });
        if text.is_none() {
            tracing::warn!("not found");
        }

        Ok(Self { text })
    }

    fn fetch_span(_request: &Self::Request) -> tracing::Span {
        info_span!("deriving_flavor_text")
    }

    fn poll(&mut self, _cx: &mut Context<'_>) -> Poll<()> {
        Poll::Pending
    }
}
