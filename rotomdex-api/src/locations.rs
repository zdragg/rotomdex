pub mod location {
    crate::endpoint!(crate::model::locations::Location; for "location");
}

pub mod location_area {
    crate::endpoint!(crate::model::locations::LocationArea; for "location-area");
}

pub mod pal_park_area {
    crate::endpoint!(crate::model::locations::PalParkArea; for "pal-park-area");
}

pub mod region {
    crate::endpoint!(crate::model::locations::Region; for "region");
}
