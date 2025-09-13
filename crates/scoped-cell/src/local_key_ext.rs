use core::{
    fmt::Debug,
    future::{Future, IntoFuture},
    pin::Pin,
    task::{Context, Poll},
};
use std::thread::LocalKey;

use crate::{ScopeIterator, ScopedCell};

/// Extension trait for `LocalKey<ScopedCell<T>>`.
pub trait ScopedCellLocalKeyExt<T: ?Sized> {
    /// See [`ScopedCell::scope`].
    fn scope<F, R>(&'static self, value: &T, f: F) -> R
    where
        F: FnOnce() -> R;

    /// See [`ScopedCell::scope_async`].
    fn scope_async<'a, F, Fut>(
        &'static self,
        value: &'a T,
        f: F,
    ) -> LocalKeyScopeFuture<'a, T, Fut::IntoFuture>
    where
        F: FnOnce() -> Fut,
        Fut: IntoFuture;

    /// See [`ScopedCell::scope_future`].
    fn scope_future<'a, Fut>(
        &'static self,
        value: &'a T,
        future: Fut,
    ) -> LocalKeyScopeFuture<'a, T, Fut::IntoFuture>
    where
        Fut: IntoFuture;

    #[cfg(feature = "futures-core")]
    /// See [`ScopedCell::scope_stream`].
    fn scope_stream<'a, S>(&'static self, value: &'a T, stream: S) -> LocalKeyScopeStream<'a, T, S>
    where
        S: futures_core::stream::Stream;

    /// See [`ScopedCell::peek`].
    fn peek<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(Option<&T>) -> R;

    /// See [`ScopedCell::iter`].
    fn iter<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(ScopeIterator<T>) -> R;
}

impl<T: ?Sized> ScopedCellLocalKeyExt<T> for LocalKey<ScopedCell<T>> {
    fn scope<F, R>(&'static self, value: &T, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.with(|cell| cell.scope(value, f))
    }

    fn scope_async<'a, F, Fut>(
        &'static self,
        value: &'a T,
        f: F,
    ) -> LocalKeyScopeFuture<'a, T, Fut::IntoFuture>
    where
        F: FnOnce() -> Fut,
        Fut: IntoFuture,
    {
        let inner = self.scope(value, f).into_future();
        self.scope_future(value, inner)
    }

    fn scope_future<'a, Fut>(
        &'static self,
        value: &'a T,
        future: Fut,
    ) -> LocalKeyScopeFuture<'a, T, Fut::IntoFuture>
    where
        Fut: IntoFuture,
    {
        LocalKeyScopeFuture {
            local_key: self,
            value,
            inner: future.into_future(),
        }
    }

    #[cfg(feature = "futures-core")]
    fn scope_stream<'a, S>(&'static self, value: &'a T, stream: S) -> LocalKeyScopeStream<'a, T, S>
    where
        S: futures_core::stream::Stream,
    {
        LocalKeyScopeStream {
            local_key: self,
            value,
            inner: stream,
        }
    }

    fn peek<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(Option<&T>) -> R,
    {
        self.with(|cell| cell.peek(f))
    }

    fn iter<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(ScopeIterator<T>) -> R,
    {
        self.with(|cell| cell.iter(f))
    }
}

/// Future returned by [`ScopedCellLocalKeyExt::scope_async`] and [`ScopedCellLocalKeyExt::scope_future`].
pub struct LocalKeyScopeFuture<'a, T: ?Sized + 'static, Fut: ?Sized> {
    local_key: &'static LocalKey<ScopedCell<T>>,
    value: &'a T,
    inner: Fut,
}

impl<T: ?Sized, Fut: ?Sized + Future> Future for LocalKeyScopeFuture<'_, T, Fut> {
    type Output = Fut::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: `inner` is always structurally pinned, whereas `local_key` and `value` are
        // never wrapped in a `Pin`.
        let (local_key, value, inner) = unsafe {
            let this = self.get_unchecked_mut();
            (
                this.local_key,
                this.value,
                Pin::new_unchecked(&mut this.inner),
            )
        };

        local_key.scope(value, || inner.poll(cx))
    }
}

#[cfg(feature = "futures-core")]
impl<T: ?Sized, Fut: ?Sized + futures_core::FusedFuture> futures_core::FusedFuture
    for LocalKeyScopeFuture<'_, T, Fut>
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

impl<T: ?Sized, Fut: Copy> Copy for LocalKeyScopeFuture<'_, T, Fut> {}

impl<T: ?Sized, Fut: Clone> Clone for LocalKeyScopeFuture<'_, T, Fut> {
    fn clone(&self) -> Self {
        Self {
            local_key: self.local_key,
            value: self.value,
            inner: self.inner.clone(),
        }
    }
}

impl<T: ?Sized + Debug, Fut: ?Sized + Debug> Debug for LocalKeyScopeFuture<'_, T, Fut> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LocalKeyScopeFuture")
            .field("value", &self.value)
            .field("inner", &&self.inner)
            .finish()
    }
}

#[cfg(feature = "futures-core")]
/// Stream returned by [`ScopedCellLocalKeyExt::scope_stream`].
pub struct LocalKeyScopeStream<'a, T: ?Sized + 'static, S: ?Sized> {
    local_key: &'static LocalKey<ScopedCell<T>>,
    value: &'a T,
    inner: S,
}

#[cfg(feature = "futures-core")]
impl<T: ?Sized, S: ?Sized + futures_core::Stream> futures_core::Stream
    for LocalKeyScopeStream<'_, T, S>
{
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // SAFETY: `inner` is always structurally pinned, whereas `local_key` and `value` are
        // never wrapped in a `Pin`.
        let (local_key, value, inner) = unsafe {
            let this = self.get_unchecked_mut();
            (
                this.local_key,
                this.value,
                Pin::new_unchecked(&mut this.inner),
            )
        };

        local_key.scope(value, || inner.poll_next(cx))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

#[cfg(feature = "futures-core")]
impl<T: ?Sized, S: ?Sized + futures_core::FusedStream> futures_core::FusedStream
    for LocalKeyScopeStream<'_, T, S>
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(feature = "futures-core")]
impl<T: ?Sized, S: Copy> Copy for LocalKeyScopeStream<'_, T, S> {}

#[cfg(feature = "futures-core")]
impl<T: ?Sized, S: Clone> Clone for LocalKeyScopeStream<'_, T, S> {
    fn clone(&self) -> Self {
        Self {
            local_key: self.local_key,
            value: self.value,
            inner: self.inner.clone(),
        }
    }
}
