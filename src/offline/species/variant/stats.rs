use color_eyre::eyre::{Result, eyre};
use rustemon::model::pokemon::PokemonStat;

#[derive(Debug, Clone)]
pub struct OfflineStats {
    pub hp: u32,
    pub atk: u32,
    pub def: u32,
    pub spa: u32,
    pub spd: u32,
    pub spe: u32,
}

impl OfflineStats {
    pub fn new(value: &[PokemonStat]) -> Result<Self> {
        let mut stats: [Option<u32>; 6] = [None; 6];
        for stat in value {
            let stat_index = match stat.stat.name.as_str() {
                "hp" => 0,
                "attack" => 1,
                "defense" => 2,
                "special-attack" => 3,
                "special-defense" => 4,
                "speed" => 5,
                _ => return Err(eyre!("invalid stat name found")),
            };
            stats[stat_index] = Some(stat.base_stat as u32);
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
    pub fn highest(&self) -> u32 {
        self.hp
            .max(self.atk)
            .max(self.def)
            .max(self.spa)
            .max(self.spd)
            .max(self.spe)
    }
}
