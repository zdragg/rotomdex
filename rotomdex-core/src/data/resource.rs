use core::{
    cell::{Cell, RefCell},
    cmp::Ordering,
    fmt,
    task::{Context, Poll, Waker},
};

use alloc::boxed::Box;
use futures::future::LocalBoxFuture;
use rotomdex_api::Client;
use snafu::Snafu;
use tracing::{Instrument, Span};

use crate::Settings;

pub(crate) type ResourceResult<T> = core::result::Result<T, ResourceError>;

#[derive(Debug, Snafu)]
pub(crate) enum ResourceError {
    #[snafu(transparent)]
    ApiError { source: rotomdex_api::Error },

    #[snafu(transparent)]
    Arbitrary {
        source: Box<dyn core::error::Error + Send + Sync + 'static>,
    },
}

pub(crate) trait ArbitraryResourceError: core::error::Error + Send + Sync + 'static {}

impl<E: ArbitraryResourceError> From<E> for ResourceError {
    fn from(value: E) -> Self {
        Self::Arbitrary {
            source: Box::new(value),
        }
    }
}

pub(crate) trait Derivable: Sized {
    type Request;
    fn derive(request: Self::Request, client: &Client, settings: Settings) -> ResourceResult<Self>;

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()>;

    fn fetch_span(request: &Self::Request) -> Span;
}

pub(crate) enum SyncResource<T: Derivable> {
    Loaded(T),
    Failed(ResourceError),
}

impl<T: Derivable> SyncResource<T> {
    pub(crate) fn derive(request: T::Request, client: &Client, settings: Settings) -> Self {
        let span = T::fetch_span(&request);
        let _span_guard = span.enter();

        let result = T::derive(request, client, settings);

        match result {
            Ok(item) => Self::Loaded(item),
            Err(report) => Self::Failed(report),
        }
    }

    pub(crate) fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        match self {
            Self::Loaded(t) => t.poll(cx),
            Self::Failed(_) => Poll::Pending,
        }
    }

    pub(crate) fn as_loaded(&self) -> Option<&T> {
        match self {
            Self::Loaded(t) => Some(t),
            Self::Failed(_) => None,
        }
    }
}

pub(crate) trait Fetchable: Sized + 'static {
    type Request;
    async fn fetch(
        request: Self::Request,
        client: Client,
        settings: Settings,
    ) -> ResourceResult<Self>;

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()>;

    fn fetch_span(request: &Self::Request) -> Span;
}

impl<T: fmt::Debug + Derivable> fmt::Debug for SyncResource<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Loaded(value) => f.debug_tuple("Loaded").field(value).finish(),
            Self::Failed(error) => f.debug_tuple("Failed").field(error).finish(),
        }
    }
}

pub(crate) enum AsyncResource<T: Fetchable> {
    Loading {
        deferred: Cell<bool>,
        deferred_waker: RefCell<Option<Waker>>,
        future: LocalBoxFuture<'static, ResourceResult<T>>,
    },
    Loaded(T),
    Failed(ResourceError),
}

impl<T: Fetchable> AsyncResource<T> {
    pub(crate) fn fetch(request: T::Request, client: &Client, settings: Settings) -> Self {
        let client = Client {
            transport: client.transport.clone(),
        };
        let span = T::fetch_span(&request);

        let future = async move {
            let result = T::fetch(request, client, settings).await;
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

    pub(crate) fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        match self {
            Self::Loading {
                deferred,
                deferred_waker,
                future,
            } => {
                if deferred.get() {
                    *deferred_waker.borrow_mut() = Some(cx.waker().clone());
                    return Poll::Pending;
                }
                let Poll::Ready(result) = future.as_mut().poll(cx) else {
                    return Poll::Pending;
                };
                *self = match result {
                    Ok(value) => Self::Loaded(value),
                    Err(error) => Self::Failed(error),
                };
                Poll::Ready(())
            }
            Self::Loaded(value) => value.poll(cx),
            Self::Failed(_) => Poll::Pending,
        }
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

    /// Accesses inner resource while keeping defer status.
    pub(crate) fn as_loaded_defer(&self) -> Option<&T> {
        if let Self::Loaded(inner) = self {
            Some(inner)
        } else {
            None
        }
    }
}

impl<T: fmt::Debug + Fetchable> fmt::Debug for AsyncResource<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Loading { deferred, .. } if deferred.get() => f.write_str("Deferred"),
            Self::Loading { .. } => f.write_str("Loading"),
            Self::Loaded(value) => f.debug_tuple("Loaded").field(value).finish(),
            Self::Failed(error) => f.debug_tuple("Failed").field(error).finish(),
        }
    }
}

impl<T: PartialEq + Fetchable> PartialEq for AsyncResource<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self.as_loaded_defer(), other.as_loaded_defer()) {
            (Some(me), Some(other)) => me == other,
            (None, None) => true,
            _ => false,
        }
    }
}

/// Mark all unloaded resources as greater. When sorting, these go to the back.
impl<T: PartialOrd + Fetchable> PartialOrd for AsyncResource<T> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        match (self.as_loaded_defer(), other.as_loaded_defer()) {
            (Some(me), Some(other)) => me.partial_cmp(other),
            (Some(_me), None) => Some(Ordering::Less),
            (None, Some(_other)) => Some(Ordering::Greater),
            (None, None) => Some(Ordering::Equal),
        }
    }
}

impl<T: Eq + Fetchable> Eq for AsyncResource<T> {}

/// Mark all unloaded resources as greater. When sorting, these go to the back.
impl<T: Ord + Fetchable> Ord for AsyncResource<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.as_loaded_defer(), other.as_loaded_defer()) {
            (Some(me), Some(other)) => me.cmp(other),
            (Some(_me), None) => Ordering::Less,
            (None, Some(_other)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        }
    }
}
