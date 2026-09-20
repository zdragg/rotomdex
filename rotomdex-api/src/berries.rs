pub mod berry {
    crate::endpoint!(crate::model::berries::Berry; for "berry");
}

pub mod berry_firmness {
    crate::endpoint!(crate::model::berries::BerryFirmness; for "berry-firmness");
}

pub mod berry_flavor {
    crate::endpoint!(crate::model::berries::BerryFlavor; for "berry-flavor");
}
