use super::{games::VersionGroup, items::Item, moves::Move, resource::NamedApiResource};

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Machine {
    pub id: i64,
    pub item: NamedApiResource<Item>,
    #[serde(rename = "move")]
    pub move_: NamedApiResource<Move>,
    pub version_group: NamedApiResource<VersionGroup>,
}
