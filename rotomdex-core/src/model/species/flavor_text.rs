use itertools::Itertools;
use rustemon::model::resource::FlavorText;

use crate::Version;

#[derive(Debug)]
pub(crate) struct ModelFlavorText {
    pub(crate) text: String,
}

impl ModelFlavorText {
    pub(super) fn new(entries: Vec<FlavorText>, target_version: Version) -> Option<Self> {
        let maybe_text = entries.into_iter().find_map(|entry| {
            if entry.version?.name.parse::<Version>().ok()? != target_version {
                return None;
            }

            if entry.language.name != "en" {
                return None;
            }

            let text = entry.flavor_text.split_whitespace().join(" ");

            Some(Self { text })
        });
        if maybe_text.is_none() {
            tracing::warn!("relevant flavor text not found");
        }
        maybe_text
    }
}
