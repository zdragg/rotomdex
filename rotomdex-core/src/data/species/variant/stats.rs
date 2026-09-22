use core::task::{Context, Poll};

use alloc::{string::String, vec::Vec};
use itertools::Itertools;
use rotomdex_api::{
    Client,
    model::pokemon::{PokemonStat, PokemonStatPast},
};
use snafu::Snafu;
use tracing::info_span;

use crate::{
    Generation,
    data::resource::{ArbitraryResourceError, Derivable, ResourceResult},
};

#[derive(Debug, Clone)]
pub(crate) struct ModelStats {
    pub(crate) hp: u32,
    pub(crate) atk: u32,
    pub(crate) def: u32,
    pub(crate) spa: u32,
    pub(crate) spd: u32,
    pub(crate) spe: u32,
}

#[derive(Debug, Snafu)]
enum StatsError {
    #[snafu(display("{name} is an invalid stat name"))]
    InvalidStatName { name: String },
    #[snafu(display(
        "stats are missing, available stats: hp: {hp}, atk: {atk}, def: {def}, spa: {spa}, spd: {spd}, spe: {spe}"
    ))]
    MissingStats {
        hp: bool,
        atk: bool,
        def: bool,
        spa: bool,
        spd: bool,
        spe: bool,
    },
}

impl ArbitraryResourceError for StatsError {}

impl Derivable for ModelStats {
    type Request = (Vec<PokemonStat>, Vec<PokemonStatPast>);
    fn derive(
        request: Self::Request,
        _client: &Client,
        settings: crate::Settings,
    ) -> ResourceResult<Self> {
        let (current, past) = request;
        let mut stats: [Option<u32>; 6] = [None; 6];
        let mut apply_stat = |stat: PokemonStat| -> ResourceResult<()> {
            let name = stat.stat.name;
            let stat_index = match name.as_str() {
                "hp" => 0,
                "attack" => 1,
                "defense" => 2,
                "special-attack" => 3,
                "special-defense" => 4,
                "speed" => 5,
                "special" => 6,
                _ => InvalidStatNameSnafu { name }.fail()?,
            };
            // Gen 1 stat patch's "special" is applied to both SpA and SpD
            if stat_index == 6 {
                stats[3] = Some(stat.base_stat as u32);
                stats[4] = Some(stat.base_stat as u32);
            } else {
                stats[stat_index] = Some(stat.base_stat as u32);
            }
            Ok(())
        };

        for stat in current {
            apply_stat(stat)?;
        }
        let target_generation = settings.version.generation();
        let patches: Vec<_> = past
            .into_iter()
            .filter_map(|stat_patch| {
                let generation = stat_patch.generation.name.parse::<Generation>().ok()?;
                (generation >= target_generation).then_some((generation, stat_patch))
            })
            .sorted_unstable_by_key(|(generation, _)| core::cmp::Reverse(*generation))
            .map(|(_, stat_patch)| stat_patch.stats)
            .collect();

        for patch in patches {
            for stat in patch {
                apply_stat(stat)?;
            }
        }

        let [
            Some(hp),
            Some(atk),
            Some(def),
            Some(spa),
            Some(spd),
            Some(spe),
        ] = stats
        else {
            MissingStatsSnafu {
                hp: stats[0].is_some(),
                atk: stats[1].is_some(),
                def: stats[2].is_some(),
                spa: stats[3].is_some(),
                spd: stats[4].is_some(),
                spe: stats[5].is_some(),
            }
            .fail()?
        };

        Ok(Self {
            hp,
            atk,
            def,
            spa,
            spd,
            spe,
        })
    }

    fn fetch_span(_request: &Self::Request) -> tracing::Span {
        info_span!("derive_stats")
    }

    fn poll(&mut self, _cx: &mut Context<'_>) -> Poll<()> {
        Poll::Pending
    }
}

impl ModelStats {
    pub(crate) fn highest(&self) -> u32 {
        self.hp
            .max(self.atk)
            .max(self.def)
            .max(self.spa)
            .max(self.spd)
            .max(self.spe)
    }
}
