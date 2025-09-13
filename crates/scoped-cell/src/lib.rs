//! A thread local cell scoped with values scoped to stack frames.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
mod local_key_ext;

use core::{
    cell::Cell,
    fmt::{self, Debug},
    future::{Future, IntoFuture},
    iter::{self, FusedIterator},
    pin::Pin,
    ptr,
    task::{Context, Poll},
};

use drop_guard::guard;
use manual_ref::ManualRef;

#[cfg(feature = "std")]
pub use crate::local_key_ext::*;

/// A thread local cell scoped with values scoped to stack frames.
pub struct ScopedCell<T: ?Sized> {
    head: Cell<Option<ManualRef<Node<T>>>>,
}

impl<T: ?Sized> ScopedCell<T> {
    /// Create a new, empty `ScopedCell`.
    pub const fn new() -> Self {
        ScopedCell {
            head: Cell::new(None),
        }
    }

    /// Run a closure with a value stored in the cell.
    ///
    /// Within the closure, `value` is accessible through [`ScopedCell::peek`].
    pub fn scope<F, R>(&self, value: &T, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let head = self.head.get();

        // SAFETY: `value` is valid for the lifetime of this call, and the
        // `ManualRef` is dropped before this call returns.
        let value = unsafe { ManualRef::new(value) };

        let node = Node {
            next: head,
            stack_size: head.map(|node| node.stack_size).unwrap_or(0) + 1,
            value,
        };

        // SAFETY: `node` is valid for the lifetime of this call, and the
        // `ManualRef` is dropped before this call returns.
        let node = unsafe { ManualRef::new(&node) };

        self.head.set(Some(node));

        guard!(f(), 'finally: { self.head.set(head) })
    }

    /// Run an asynchronous closure with a value stored in the cell.
    ///
    /// Within the closure or its future, `value` is accessible through
    /// [`ScopedCell::peek`].
    pub fn scope_async<'a, F, Fut>(
        &'a self,
        value: &'a T,
        f: F,
    ) -> ScopeFuture<'a, T, Fut::IntoFuture>
    where
        F: FnOnce() -> Fut,
        Fut: IntoFuture,
    {
        let fut = self.scope(value, f);
        self.scope_future(value, fut)
    }

    /// Run a future with a value stored in the cell.
    ///
    /// Within the future, `value` is accessible through
    /// [`ScopedCell::peek`].
    pub fn scope_future<'a, Fut>(
        &'a self,
        value: &'a T,
        fut: Fut,
    ) -> ScopeFuture<'a, T, Fut::IntoFuture>
    where
        Fut: IntoFuture,
    {
        ScopeFuture {
            cell: self,
            value,
            inner: fut.into_future(),
        }
    }

    /// Run a stream with a value stored in the cell.
    ///
    /// Within the stream, `value` is accessible through
    /// [`ScopedCell::peek`].
    #[cfg(feature = "futures-core")]
    pub fn scope_stream<'a, S>(&'a self, value: &'a T, stream: S) -> ScopeStream<'a, T, S>
    where
        S: futures_core::stream::Stream,
    {
        ScopeStream {
            cell: self,
            value,
            inner: stream,
        }
    }

    /// Peek at the value stored in the cell.
    pub fn peek<F, R>(&self, f: F) -> R
    where
        F: FnOnce(Option<&T>) -> R,
    {
        let head = self.head.get();
        f(head.as_ref().map(|node| &*node.value))
    }

    /// Access an iterator over the values on the stack.
    ///
    /// The first value in the iterator is the value most recently passed to
    /// [`ScopedCell::scope`], i.e. the value given by [`ScopedCell::peek`].
    pub fn iter<F, R>(&self, f: F) -> R
    where
        F: FnOnce(ScopeIterator<'_, T>) -> R,
    {
        f(ScopeIterator {
            head: self.head.get().as_deref(),
            tail: None,
        })
    }
}

impl<T: ?Sized> Default for ScopedCell<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized + Debug> Debug for ScopedCell<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.iter(|iter| f.debug_tuple("ScopedCell").field(&iter).finish())
    }
}

/// Future returned by [`ScopedCell::scope_async`] and
/// [`ScopedCell::scope_future`].
pub struct ScopeFuture<'a, T: ?Sized, Fut: ?Sized> {
    cell: &'a ScopedCell<T>,
    value: &'a T,
    inner: Fut,
}

impl<T: ?Sized, Fut: ?Sized + Future> Future for ScopeFuture<'_, T, Fut> {
    type Output = Fut::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: `inner` is always structurally pinned, whereas `cell` and `value` are
        // never wrapped in a `Pin`.
        let (cell, value, inner) = unsafe {
            let this = self.get_unchecked_mut();
            (this.cell, this.value, Pin::new_unchecked(&mut this.inner))
        };

        cell.scope(value, || inner.poll(cx))
    }
}

#[cfg(feature = "futures-core")]
impl<T: ?Sized, Fut: ?Sized + futures_core::FusedFuture> futures_core::FusedFuture
    for ScopeFuture<'_, T, Fut>
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

impl<T: ?Sized, Fut: Copy> Copy for ScopeFuture<'_, T, Fut> {}

impl<T: ?Sized, Fut: Clone> Clone for ScopeFuture<'_, T, Fut> {
    fn clone(&self) -> Self {
        Self {
            cell: self.cell,
            value: self.value,
            inner: self.inner.clone(),
        }
    }
}

impl<T: Debug + ?Sized, Fut: ?Sized + Debug> Debug for ScopeFuture<'_, T, Fut> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScopeFuture")
            .field("value", &self.value)
            .field("inner", &&self.inner)
            .finish()
    }
}

#[cfg(feature = "futures-core")]
/// Stream returned by [`ScopedCell::scope_stream`].
pub struct ScopeStream<'a, T: ?Sized, S: ?Sized> {
    cell: &'a ScopedCell<T>,
    value: &'a T,
    inner: S,
}

#[cfg(feature = "futures-core")]
impl<T: ?Sized, S: ?Sized + futures_core::Stream> futures_core::Stream for ScopeStream<'_, T, S> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // SAFETY: `inner` is always structurally pinned, whereas `cell` and `value` are
        // never wrapped in a `Pin`.
        let (cell, value, inner) = unsafe {
            let this = self.get_unchecked_mut();
            (this.cell, this.value, Pin::new_unchecked(&mut this.inner))
        };

        cell.scope(value, || inner.poll_next(cx))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

#[cfg(feature = "futures-core")]
impl<T: ?Sized, S: ?Sized + futures_core::FusedStream> futures_core::FusedStream
    for ScopeStream<'_, T, S>
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(feature = "futures-core")]
impl<T: ?Sized, S: Copy> Copy for ScopeStream<'_, T, S> {}

#[cfg(feature = "futures-core")]
impl<T: ?Sized, S: Clone> Clone for ScopeStream<'_, T, S> {
    fn clone(&self) -> Self {
        Self {
            cell: self.cell,
            value: self.value,
            inner: self.inner.clone(),
        }
    }
}

#[cfg(feature = "futures-core")]
impl<T: Debug + ?Sized, S: Debug + ?Sized> Debug for ScopeStream<'_, T, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScopeStream")
            .field("value", &self.value)
            .field("inner", &&self.inner)
            .finish()
    }
}

/// An iterator over the values on the stack.
///
/// The first value in the iterator is the topmost value of the stack, i.e. the
/// value most recently passed to [`ScopedCell::scope`]. The last value in the
/// iterator is the bottommost value of the stack, i.e. the value passed to the
/// first call to [`ScopedCell::scope`].
pub struct ScopeIterator<'a, T: ?Sized> {
    head: Option<&'a Node<T>>,
    tail: Option<&'a Node<T>>,
}

impl<T: ?Sized> Copy for ScopeIterator<'_, T> {}

impl<T: ?Sized> Clone for ScopeIterator<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T: ?Sized> Iterator for ScopeIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        let head = self.head?;

        if let Some(tail) = self.tail
            && ptr::eq(head, tail)
        {
            self.head = None;
            self.tail = None;
            return None;
        }

        let value = &*head.value;
        self.head = head.next.as_deref();
        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let head_size = self.head.map(|node| node.stack_size).unwrap_or(0);
        let tail_size = self.tail.map(|node| node.stack_size).unwrap_or(0);

        let size = head_size - tail_size;
        (size, Some(size))
    }
}

impl<'a, T: ?Sized> DoubleEndedIterator for ScopeIterator<'a, T> {
    fn next_back(&mut self) -> Option<&'a T> {
        let prev = iter::successors(self.head, |node| node.next.as_deref()).find(|node| {
            match (node.next, self.tail) {
                (Some(next), Some(tail)) => ptr::eq(&*next, tail),
                (None, None) => true,
                _ => false,
            }
        });

        let prev = match prev {
            Some(prev) => prev,
            None => {
                self.head = None;
                self.tail = None;
                return None;
            }
        };

        self.tail = Some(prev);
        Some(&*prev.value)
    }
}

impl<T: ?Sized> ExactSizeIterator for ScopeIterator<'_, T> {}

impl<T: ?Sized> FusedIterator for ScopeIterator<'_, T> {}

impl<T: ?Sized + Debug> Debug for ScopeIterator<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(*self).finish()
    }
}

struct Node<T: ?Sized> {
    next: Option<ManualRef<Node<T>>>,
    stack_size: usize,
    value: ManualRef<T>,
}
