use alloc::{string::String, vec::Vec};

use super::resource::Name;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Language {
    pub id: i64,
    pub name: String,
    pub official: bool,
    pub iso639: String,
    pub iso3166: String,
    pub names: Vec<Name>,
}
