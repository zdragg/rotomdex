pub mod encounter_method {
    crate::endpoint!(crate::model::encounters::EncounterMethod; for "encounter-method");
}

pub mod encounter_condition {
    crate::endpoint!(crate::model::encounters::EncounterCondition; for "encounter-condition");
}

pub mod encounter_condition_value {
    crate::endpoint!(crate::model::encounters::EncounterConditionValue; for "encounter-condition-value");
}
