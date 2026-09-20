pub mod item {
    crate::endpoint!(crate::model::items::Item; for "item");
}

pub mod item_attribute {
    crate::endpoint!(crate::model::items::ItemAttribute; for "item-attribute");
}

pub mod item_category {
    crate::endpoint!(crate::model::items::ItemCategory; for "item-category");
}

pub mod item_fling_effect {
    crate::endpoint!(crate::model::items::ItemFlingEffect; for "item-fling-effect");
}

pub mod item_pocket {
    crate::endpoint!(crate::model::items::ItemPocket; for "item-pocket");
}
