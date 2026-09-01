use color_eyre::eyre::{Result, eyre};
use itertools::Itertools;
use rustemon::model::pokemon::{PokemonStat, PokemonStatPast};

use crate::{Generation, ModelContext};

#[derive(Debug, Clone)]
pub(crate) struct ModelStats {
    pub(crate) hp: u32,
    pub(crate) atk: u32,
    pub(crate) def: u32,
    pub(crate) spa: u32,
    pub(crate) spd: u32,
    pub(crate) spe: u32,
}

impl ModelStats {
    pub(crate) fn new(current: &[PokemonStat], past: &[PokemonStatPast], ctx: ModelContext) -> Result<Self> {
        let mut stats: [Option<u32>; 6] = [None; 6];
        let mut apply_stat = |stat: &PokemonStat| {
            let stat_index = match stat.stat.name.as_str() {
                "hp" => 0,
                "attack" => 1,
                "defense" => 2,
                "special-attack" => 3,
                "special-defense" => 4,
                "speed" => 5,
                "special" => 6,
                _ => return Err(eyre!("invalid stat name found")),
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
        let target_generation = ctx.version.generation();
        let patches: Vec<_> = past
            .iter()
            .filter_map(|stat_patch| {
                let generation = stat_patch.generation.name.parse::<Generation>().ok()?;
                (generation >= target_generation).then_some((generation, stat_patch))
            })
            .sorted_unstable_by_key(|(generation, _)| std::cmp::Reverse(*generation))
            .map(|(_, stat_patch)| &stat_patch.stats[..])
            .collect();

        for patch in patches {
            for stat in patch {
                apply_stat(stat)?;
            }
        }

        let [Some(hp), Some(atk), Some(def), Some(spa), Some(spd), Some(spe)] = stats else {
            return Err(eyre!("missing stat"));
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

    pub(crate) fn highest(&self) -> u32 {
        self.hp
            .max(self.atk)
            .max(self.def)
            .max(self.spa)
            .max(self.spd)
            .max(self.spe)
    }
}
