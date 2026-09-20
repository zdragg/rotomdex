pub mod move_ {
    crate::endpoint!(crate::model::moves::Move; for "move");
}

pub mod move_ailment {
    crate::endpoint!(crate::model::moves::MoveAilment; for "move-ailment");
}

pub mod move_battle_style {
    crate::endpoint!(crate::model::moves::MoveBattleStyle; for "move-battle-style");
}

pub mod move_category {
    crate::endpoint!(crate::model::moves::MoveCategory; for "move-category");
}

pub mod move_damage_class {
    crate::endpoint!(crate::model::moves::MoveDamageClass; for "move-damage-class");
}

pub mod move_learn_method {
    crate::endpoint!(crate::model::moves::MoveLearnMethod; for "move-learn-method");
}

pub mod move_target {
    crate::endpoint!(crate::model::moves::MoveTarget; for "move-target");
}
