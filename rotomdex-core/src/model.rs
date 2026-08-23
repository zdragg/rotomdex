mod species;
pub(crate) use species::*;

use std::{
    fmt,
    task::{Context, Poll},
};

use color_eyre::eyre::{Report, Result};
use futures::future::LocalBoxFuture;
use web_time::Instant;

use crate::FetchContext;

pub(crate) struct ModelPokemon {
    name: String,
    species: Resource<ModelSpecies>,
    benchmark: Instant,
    loaded: bool,
}

impl ModelPokemon {
    pub(crate) fn new(name: impl Into<String>, ctx: FetchContext) -> Self {
        let name = name.into();
        let result = Self {
            name: name.clone(),

            benchmark: Instant::now(),

            species: Resource::<ModelSpecies>::fetch(name, ctx),

            loaded: false,
        };
        result
    }

    pub(crate) async fn poll(&mut self) {
        std::future::poll_fn(|cx| self.species.poll(cx)).await;
        if !self.loaded && self.is_loaded() {
            self.loaded = true;
            log::info!(
                "{} fully loaded in {}ms",
                self.name,
                self.benchmark.elapsed().as_millis()
            );
        }
    }

    pub(crate) fn species(&self) -> &Resource<ModelSpecies> {
        &self.species
    }

    pub(crate) fn is_loaded(&self) -> bool {
        self.species.is_loaded()
    }
}

pub(crate) trait Fetchable: Sized + 'static {
    type Request: 'static;
    async fn fetch(request: Self::Request, ctx: FetchContext) -> Result<Self>;

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()>;

    fn is_loaded(&self) -> bool;
}

pub(crate) enum Resource<T: Fetchable> {
    Loading(LocalBoxFuture<'static, Result<T>>),
    Loaded(T),
    Failed(Report),
}

impl<T: Fetchable> Resource<T> {
    pub(crate) fn fetch(request: T::Request, ctx: FetchContext) -> Self {
        Self::Loading(Box::pin(T::fetch(request, ctx)))
    }

    pub(crate) fn as_loaded(&self) -> Option<&T> {
        if let Self::Loaded(inner) = self {
            Some(inner)
        } else {
            None
        }
    }

    pub(crate) fn is_loaded(&self) -> bool {
        if let Self::Loaded(value) = self {
            return value.is_loaded();
        }
        false
    }

    pub(crate) fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        let result = match self {
            Self::Loading(future) => match future.as_mut().poll(cx) {
                Poll::Ready(result) => result,
                Poll::Pending => return Poll::Pending,
            },
            Self::Loaded(value) => return value.poll(cx),
            _ => return Poll::Pending,
        };

        *self = match result {
            Ok(value) => Self::Loaded(value),
            Err(error) => {
                log::error!("{error}");
                Self::Failed(error)
            }
        };

        Poll::Ready(())
    }
}

impl<T: fmt::Debug + Fetchable> fmt::Debug for Resource<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Loading(_) => f.write_str("Loading"),
            Self::Loaded(value) => f.debug_tuple("Loaded").field(value).finish(),
            Self::Failed(error) => f.debug_tuple("Failed").field(error).finish(),
        }
    }
}
