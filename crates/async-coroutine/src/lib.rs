//! Asynchronous version of [`Coroutine`](crate::compat::Coroutine).

#![no_std]
#![cfg_attr(feature = "nightly", feature(coroutine_trait))]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::{
    future::poll_fn,
    pin::{Pin, pin},
    task::{Context, Poll},
};

#[cfg(feature = "nightly")]
pub use core::ops::CoroutineState;

#[cfg(not(feature = "nightly"))]
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum CoroutineState<Y, R> {
    Yielded(Y),
    Complete(R),
}

/// Asynchronous version of [`Coroutine`](crate::compat::Coroutine).
#[allow(missing_docs)]
pub trait AsyncCoroutine<R = ()> {
    type Yield;
    type Return;

    /// Resume the coroutine.
    ///
    /// After this method has been called, it may not be called again until
    /// [`poll_resume`](AsyncCoroutine::poll_resume) returns `Poll::Ready`.
    ///
    /// This method may not be called after `poll_resume` returns
    /// `CoroutineState::Complete`.
    fn start_resume(self: Pin<&mut Self>, arg: R);

    /// Poll until the coroutine yields or completes.
    ///
    /// This method must only be called after [`start_resume`](AsyncCoroutine::start_resume)
    /// returns. After it returns `Poll::Ready`, it may not be called again until
    /// `start_resume` calls again.
    fn poll_resume(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<CoroutineState<Self::Yield, Self::Return>>;
}

impl<G: ?Sized + AsyncCoroutine<R> + Unpin, R> AsyncCoroutine<R> for &mut G {
    type Yield = G::Yield;
    type Return = G::Return;

    fn start_resume(self: Pin<&mut Self>, arg: R) {
        let this = &mut **Pin::into_inner(self);
        G::start_resume(Pin::new(this), arg)
    }

    fn poll_resume(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<CoroutineState<G::Yield, G::Return>> {
        let this = &mut **Pin::into_inner(self);
        G::poll_resume(Pin::new(this), cx)
    }
}

impl<G: ?Sized + AsyncCoroutine<R>, R> AsyncCoroutine<R> for Pin<&mut G> {
    type Yield = G::Yield;
    type Return = G::Return;

    fn start_resume(self: Pin<&mut Self>, arg: R) {
        G::start_resume(self.as_deref_mut(), arg)
    }

    fn poll_resume(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<CoroutineState<G::Yield, G::Return>> {
        G::poll_resume(self.as_deref_mut(), cx)
    }
}

#[cfg(feature = "alloc")]
impl<G: ?Sized + AsyncCoroutine<R> + Unpin, R> AsyncCoroutine<R> for alloc::boxed::Box<G> {
    type Yield = G::Yield;
    type Return = G::Return;

    fn start_resume(self: Pin<&mut Self>, arg: R) {
        let this = &mut **Pin::into_inner(self);
        G::start_resume(Pin::new(this), arg)
    }

    fn poll_resume(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<CoroutineState<G::Yield, G::Return>> {
        let this = &mut **Pin::into_inner(self);
        G::poll_resume(Pin::new(this), cx)
    }
}

#[cfg(feature = "alloc")]
impl<G: ?Sized + AsyncCoroutine<R>, R> AsyncCoroutine<R> for Pin<alloc::boxed::Box<G>> {
    type Yield = G::Yield;
    type Return = G::Return;

    fn start_resume(self: Pin<&mut Self>, arg: R) {
        G::start_resume(self.as_deref_mut(), arg)
    }

    fn poll_resume(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<CoroutineState<G::Yield, G::Return>> {
        G::poll_resume(self.as_deref_mut(), cx)
    }
}

/// Extension methods for [`AsyncCoroutine`].
pub trait AsyncCoroutineExt<R = ()>: AsyncCoroutine<R> {
    /// Resume the coroutine.
    ///
    /// This is equivalent to calling `self.start_resume(arg)` and then polling
    /// until the coroutine yields or completes.
    async fn resume(&mut self, arg: R) -> CoroutineState<Self::Yield, Self::Return>
    where
        Self: Unpin,
    {
        let mut this = Pin::new(self);

        this.as_mut().start_resume(arg);
        poll_fn(|cx| this.as_mut().poll_resume(cx)).await
    }

    /// Run the coroutine until completion.
    ///
    /// # Parameters
    ///
    /// `init_arg` specifies the initial argument to pass to the coroutine.
    ///
    /// `arg_fn` is called every time the coroutine yields, and takes in the yielded
    /// value and must return the next argument to pass to the coroutine.
    async fn run_with<F>(self, init_arg: R, mut arg_fn: F) -> Self::Return
    where
        Self: Sized,
        F: FnMut(Self::Yield) -> R,
    {
        let mut this = pin!(self);

        let mut arg = init_arg;
        loop {
            match this.resume(arg).await {
                CoroutineState::Yielded(yielded) => arg = arg_fn(yielded),
                CoroutineState::Complete(output) => return output,
            }
        }
    }

    /// Run the coroutine into completion.
    ///
    /// Equivalent to `self.run_with(R::default(), |_| R::default())`.
    #[inline(always)]
    async fn run(self) -> Self::Return
    where
        Self: Sized,
        R: Default,
    {
        self.run_with(R::default(), |_| R::default()).await
    }

    #[cfg(feature = "futures-core")]
    /// Convert the coroutine into a stream of `CoroutineState` values.
    ///
    /// # Parameters
    ///
    /// `init_arg` specifies the initial argument to pass to the coroutine.
    ///
    /// `arg_fn` is called every time the coroutine yields, and takes in the yielded
    /// value and must return the next argument to pass to the coroutine.
    fn into_state_stream_with<F>(
        self,
        init_arg: R,
        arg_fn: F,
    ) -> impl futures_core::Stream<Item = CoroutineState<Self::Yield, Self::Return>>
    where
        Self: Sized,
        F: FnMut(&Self::Yield) -> R,
    {
        #[pin_project::pin_project]
        pub struct IntoStateStreamWith<G, R, F> {
            #[pin]
            generator: G,
            next_arg: Option<R>,
            arg_fn: F,
            is_done: bool,
        }

        impl<G, R, F> futures_core::Stream for IntoStateStreamWith<G, R, F>
        where
            G: AsyncCoroutine<R>,
            F: FnMut(&G::Yield) -> R,
        {
            type Item = CoroutineState<G::Yield, G::Return>;

            fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                let mut this = self.project();

                if *this.is_done {
                    return Poll::Ready(None);
                }

                if let Some(arg) = this.next_arg.take() {
                    this.generator.as_mut().start_resume(arg);
                }

                let result = core::task::ready!(this.generator.poll_resume(cx));

                match &result {
                    CoroutineState::Yielded(yielded) => {
                        *this.next_arg = Some((this.arg_fn)(yielded));
                    }
                    CoroutineState::Complete(_) => {
                        *this.is_done = true;
                    }
                }

                Poll::Ready(Some(result))
            }
        }

        impl<G, R, F> futures_core::FusedStream for IntoStateStreamWith<G, R, F>
        where
            G: AsyncCoroutine<R>,
            F: FnMut(&G::Yield) -> R,
        {
            fn is_terminated(&self) -> bool {
                self.is_done
            }
        }

        IntoStateStreamWith {
            generator: self,
            next_arg: Some(init_arg),
            arg_fn,
            is_done: false,
        }
    }

    #[cfg(feature = "futures-core")]
    /// Convert the coroutine into a stream of `CoroutineState` values.
    ///
    /// Equivalent to `self.into_state_stream_with(R::default(), |_| R::default())`.
    #[inline(always)]
    fn into_state_stream(
        self,
    ) -> impl futures_core::Stream<Item = CoroutineState<Self::Yield, Self::Return>>
    where
        Self: Sized,
        R: Default,
    {
        self.into_state_stream_with(R::default(), |_| R::default())
    }

    #[cfg(feature = "futures")]
    /// Convert the coroutine into a stream.
    ///
    /// # Parameters
    ///
    /// `init_arg` specifies the initial argument to pass to the coroutine.
    ///
    /// `arg_fn` is called every time the coroutine yields, and takes in the yielded
    /// value and must return the next argument to pass to the coroutine.
    fn into_stream_with<F>(
        self,
        init_arg: R,
        arg_fn: F,
    ) -> impl futures_core::Stream<Item = Self::Yield>
    where
        Self: Sized,
        F: FnMut(&Self::Yield) -> R,
    {
        use core::future::ready;

        use futures::StreamExt;

        self.into_state_stream_with(init_arg, arg_fn)
            .filter_map(|state| match state {
                CoroutineState::Yielded(yielded) => ready(Some(yielded)),
                CoroutineState::Complete(_) => ready(None),
            })
    }

    #[cfg(feature = "futures")]
    /// Convert the coroutine into a stream.
    ///
    /// Equivalent to `self.into_stream_with(R::default(), |_| R::default())`.
    #[inline(always)]
    fn into_stream(self) -> impl futures_core::Stream<Item = Self::Yield>
    where
        Self: Sized,
        R: Default,
    {
        self.into_stream_with(R::default(), |_| R::default())
    }
}

impl<G: ?Sized + AsyncCoroutine<R>, R> AsyncCoroutineExt<R> for G {}
