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
    pub(crate) fn to_strings(&self) -> Vec<String> {
        let Some(detail) = &self.inner else {
            return Vec::new();
        };

        let mut requirements = vec![format!("trigger: {}", detail.trigger.name)];

        for (label, value) in [
            ("level >=", detail.min_level),
            ("happiness >=", detail.min_happiness),
            ("beauty >=", detail.min_beauty),
            ("affection >=", detail.min_affection),
            ("times move used >=", detail.min_move_count),
            ("steps walked >=", detail.min_steps),
            ("damage taken >=", detail.min_damage_taken),
        ] {
            if let Some(value) = value {
                requirements.push(format!("{label} {value}"));
            }
        }

        for (label, name) in [
            ("use item:", detail.item.as_ref().map(|item| item.name.as_str())),
            ("hold item:", detail.held_item.as_ref().map(|item| item.name.as_str())),
            (
                "knows move:",
                detail.known_move.as_ref().map(|move_| move_.name.as_str()),
            ),
            (
                "knows move with type:",
                detail.known_move_type.as_ref().map(|type_| type_.name.as_str()),
            ),
            ("use move:", detail.used_move.as_ref().map(|move_| move_.name.as_str())),
            (
                "location:",
                detail.location.as_ref().map(|location| location.name.as_str()),
            ),
            ("region:", detail.region.as_ref().map(|region| region.name.as_str())),
            (
                "species in party:",
                detail.party_species.as_ref().map(|species| species.name.as_str()),
            ),
            (
                "type in party:",
                detail.party_type.as_ref().map(|type_| type_.name.as_str()),
            ),
            (
                "trade for:",
                detail.trade_species.as_ref().map(|species| species.name.as_str()),
            ),
            (
                "starting form:",
                detail.base_form.as_ref().map(|form| form.name.as_str()),
            ),
        ] {
            if let Some(name) = name {
                requirements.push(format!("{label} {name}"));
            }
        }

        if let Some(gender) = detail.gender {
            let gender = match gender {
                1 => "gender: female".to_string(),
                2 => "gender: male".to_string(),
                3 => "gender: genderless".to_string(),
                other => format!("gender ID {other}"),
            };
            requirements.push(gender);
        }
        if let Some(stats) = detail.relative_physical_stats {
            requirements.push(match stats {
                -1 => "Attack < Defense".to_string(),
                0 => "Attack = Defense".to_string(),
                1 => "Attack > Defense".to_string(),
                other => format!("relative stats: {other}"),
            });
        }
        if !detail.time_of_day.trim().is_empty() {
            requirements.push(format!("time: {}", detail.time_of_day));
        }
        for (required, label) in [
            (detail.near_special_rock, "near a special rock"),
            (detail.needs_multiplayer, "multiplayer required"),
            (detail.needs_overworld_rain, "rain in overworld"),
            (detail.turn_upside_down, "hold 3ds upside down"),
        ] {
            if required {
                requirements.push(label.to_string());
            }
        }

        requirements
    }

    fn new(detail: Vec<EvolutionDetail>, ctx: &ModelContext) -> Result<Self> {
        let detail = detail
            .into_iter()
            .filter_map(|detail| {
                let version_group = detail.version_group.name.parse::<VersionGroup>().ok()?;
                if version_group > ctx.version.version_group() {
                    return None;
                }
                Some((version_group as usize, detail))
            })
            .max_by_key(|(version_group, _)| *version_group)
            .map(|(_, detail)| detail);

        Ok(Self { inner: detail })
    }
}
