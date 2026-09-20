pub mod ability {
    crate::endpoint!(crate::model::pokemon::Ability; for "ability");
}

pub mod characteristic {
    crate::endpoint!(unnamed crate::model::pokemon::Characteristic; for "characteristic");
}

pub mod egg_group {
    crate::endpoint!(crate::model::pokemon::EggGroup; for "egg-group");
}

pub mod gender {
    crate::endpoint!(crate::model::pokemon::Gender; for "gender");
}

pub mod growth_rate {
    crate::endpoint!(crate::model::pokemon::GrowthRate; for "growth-rate");
}

pub mod nature {
    crate::endpoint!(crate::model::pokemon::Nature; for "nature");
}

pub mod pokeathlon_stat {
    crate::endpoint!(crate::model::pokemon::PokeathlonStat; for "pokeathlon-stat");
}

#[allow(clippy::module_inception)]
pub mod pokemon {
    crate::endpoint!(crate::model::pokemon::Pokemon; for "pokemon"; with (encounters, Vec<crate::model::pokemon::LocationAreaEncounter>));
}

pub mod pokemon_color {
    crate::endpoint!(crate::model::pokemon::PokemonColor; for "pokemon-color");
}

pub mod pokemon_form {
    crate::endpoint!(crate::model::pokemon::PokemonForm; for "pokemon-form");
}

pub mod pokemon_habitat {
    crate::endpoint!(crate::model::pokemon::PokemonHabitat; for "pokemon-habitat");
}

pub mod pokemon_shape {
    crate::endpoint!(crate::model::pokemon::PokemonShape; for "pokemon-shape");
}

pub mod pokemon_species {
    crate::endpoint!(crate::model::pokemon::PokemonSpecies; for "pokemon-species");
}

pub mod stat {
    crate::endpoint!(crate::model::pokemon::Stat; for "stat");
}

pub mod type_ {
    crate::endpoint!(crate::model::pokemon::Type; for "type");
}
