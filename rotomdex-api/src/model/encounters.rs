use alloc::{string::String, vec::Vec};

use super::resource::{Name, NamedApiResource};

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct EncounterMethod {
    pub id: i64,
    pub name: String,
    pub order: i64,
    pub names: Vec<Name>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]

pub struct EncounterCondition {
    pub id: i64,
    pub name: String,
    pub names: Vec<Name>,
    pub values: Vec<NamedApiResource<EncounterConditionValue>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]

pub struct EncounterConditionValue {
    pub id: i64,
    pub name: String,
    pub condition: NamedApiResource<EncounterCondition>,
    pub names: Vec<Name>,
}
