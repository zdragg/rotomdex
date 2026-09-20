pub(crate) mod resource;
mod species;
use alloc::borrow::ToOwned;
use rotomdex_api::client::Client;
pub(crate) use species::*;

use crate::{Settings, data::resource::AsyncResource};

pub(crate) struct DataStore {
    client: Client,
    pub(crate) resource: AsyncResource<ModelSpecies>,
}

impl DataStore {
    pub(crate) fn new(client: Client, pkmn_name: &str, settings: Settings) -> Self {
        let resource =
            AsyncResource::<ModelSpecies>::fetch(pkmn_name.to_owned(), &client, settings);
        Self { client, resource }
    }

    pub(crate) fn new_resource(&mut self, pkmn_name: &str, settings: Settings) {
        self.resource =
            AsyncResource::<ModelSpecies>::fetch(pkmn_name.to_owned(), &self.client, settings)
    }

    pub(crate) async fn poll(&mut self) {
        core::future::poll_fn(|cx| self.resource.poll(cx)).await;
    }
}
