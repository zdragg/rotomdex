pub mod generation {
    crate::endpoint!(crate::model::games::Generation; for "generation");
}

pub mod pokedex {
    crate::endpoint!(crate::model::games::Pokedex; for "pokedex");
}

pub mod version {
    crate::endpoint!(crate::model::games::Version; for "version");
}

pub mod version_group {
    crate::endpoint!(crate::model::games::VersionGroup; for "version-group");
}
