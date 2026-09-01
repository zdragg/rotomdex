use rustemon::model::resource::FlavorText;

use crate::Version;

#[derive(Debug)]
pub(crate) struct ModelFlavorText {
    pub(crate) text: String,
}

impl ModelFlavorText {
    pub(super) fn new(entries: &[FlavorText], target_version: Version) -> Option<Self> {
        let maybe_text = entries.iter().find_map(|entry| {
            if entry.version.clone()?.name.parse::<Version>().ok()? != target_version {
                return None;
            }
            if entry.language.name != "en" {
                return None;
            }
            Some(Self {
                text: entry.flavor_text.clone(),
            })
        });
        if maybe_text.is_none() {
            tracing::warn!("relevant flavor text not found");
        }
        maybe_text
    }
}
