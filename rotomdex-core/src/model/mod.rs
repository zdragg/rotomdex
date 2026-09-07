mod species;
pub(crate) use species::*;

use std::{
    cell::{Cell, RefCell},
    fmt,
    task::{Context, Poll, Waker},
};

use color_eyre::eyre::{Report, Result};
use futures::future::LocalBoxFuture;
use tracing::{Instrument, Span};
use web_time::Instant;

use crate::ModelContext;

pub(crate) struct ModelPokemon {
    name: String,
    pub(crate) species: Resource<ModelSpecies>,
    benchmark: Instant,
    loaded: bool,
}

impl ModelPokemon {
    pub(crate) fn new(name: impl Into<String>, ctx: ModelContext) -> Self {
        let name = name.into();
        Self {
            name: name.clone(),

            benchmark: Instant::now(),

            species: Resource::<ModelSpecies>::fetch(name, &ctx),

            loaded: false,
        }
    }

    pub(crate) async fn poll(&mut self) {
        std::future::poll_fn(|cx| self.species.poll(cx)).await;
        if !self.loaded && self.is_loaded() {
            self.loaded = true;
            tracing::info!(
                "{} probably did not fully load in {}ms. This benchmark may be BROKEN.",
                self.name,
                self.benchmark.elapsed().as_millis()
            );
        }
    }

    pub(crate) fn is_loaded(&self) -> bool {
        self.species.is_loaded()
    }
}

pub(crate) trait Fetchable: Sized + 'static {
    type Request: 'static;
    async fn fetch(request: Self::Request, ctx: ModelContext) -> Result<Self>;

    fn is_loaded(&self) -> bool;

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()>;

    fn fetch_span(request: &Self::Request) -> Span;
}

pub(crate) enum Resource<T: Fetchable> {
    Loading {
        deferred: Cell<bool>,
        deferred_waker: RefCell<Option<Waker>>,
        future: LocalBoxFuture<'static, Result<T>>,
    },
    Loaded(T),
    Failed(Report),
}

impl<T: Fetchable> Resource<T> {
    pub(crate) fn fetch(request: T::Request, ctx: &ModelContext) -> Self {
        let ctx = ctx.clone();
        let span = T::fetch_span(&request);

        let future = async move {
            let result = T::fetch(request, ctx).await;
            if let Err(error) = &result {
                tracing::error!(error = %error, "resource fetch failed");
            }
            result
        }
        .instrument(span);

        Self::Loading {
            deferred: Cell::new(true),
            deferred_waker: RefCell::new(None),
            future: Box::pin(future),
        }
    }

    pub(crate) fn is_loaded(&self) -> bool {
        match self {
            Self::Loaded(value) => value.is_loaded(),
            Self::Loading { deferred, .. } => deferred.get(),
            _ => false,
        }
    }

    pub(crate) fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        let result = match self {
            Self::Loading {
                deferred,
                deferred_waker,
                future,
            } => {
                if deferred.get() {
                    *deferred_waker.borrow_mut() = Some(cx.waker().clone());
                    return Poll::Pending;
                }
                match future.as_mut().poll(cx) {
                    Poll::Ready(result) => result,
                    Poll::Pending => return Poll::Pending,
                }
            }
            Self::Loaded(value) => return value.poll(cx),
            Self::Failed(_) => return Poll::Pending,
        };

        *self = match result {
            Ok(value) => Self::Loaded(value),
            Err(error) => Self::Failed(error),
        };

        Poll::Ready(())
    }

    pub(crate) fn as_loaded(&self) -> Option<&T> {
        match self {
            Self::Loaded(inner) => Some(inner),
            Self::Loading {
                deferred,
                deferred_waker,
                ..
            } => {
                deferred.set(false);
                if let Some(waker) = deferred_waker.borrow_mut().take() {
                    waker.wake();
                }
                None
            }
            Self::Failed(_) => None,
        }
    }
}

impl<T: fmt::Debug + Fetchable> fmt::Debug for Resource<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Loading { deferred, .. } if deferred.get() => f.write_str("Deferred"),
            Self::Loading { .. } => f.write_str("Loading"),
            Self::Loaded(value) => f.debug_tuple("Loaded").field(value).finish(),
            Self::Failed(error) => f.debug_tuple("Failed").field(error).finish(),
        }
    }
}
