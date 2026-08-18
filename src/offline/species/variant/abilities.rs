use rustemon::model::pokemon::Ability;

#[derive(Debug)]
pub struct OfflineAbility {
    inner: Option<(String, String)>, // name, effect description
}

impl OfflineAbility {
    pub fn new(ability: Option<Ability>) -> Self {
        let Some(ability) = ability else {
            return Self { inner: None };
        };
        let name = ability.name;
        let effect_description = ability
            .effect_entries
            .into_iter()
            .find(|e| e.language.name == "en")
            .unwrap()
            .effect;
        Self {
            inner: Some((name, effect_description)),
        }
    }

    pub fn get(&self) -> Option<&(String, String)> {
        self.inner.as_ref()
    }
}
