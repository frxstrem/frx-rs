//! An asynchronous state machine implementation.
//!
//! This module provides the [`StateMachine`] struct, which can wrap a state transition
//! function and drive it to completion. It also allows for the latest state to be
//! accessed, so that the state machine can be paused and resumed later.

#![no_std]
#![cfg_attr(feature = "nightly", feature(type_alias_impl_trait, coroutine_trait))]

#[cfg(feature = "serde")]
use core::marker::PhantomData;
use core::{
    ops::ControlFlow::{self, Break, Continue},
    pin::{Pin, pin},
    task::{Context, Poll, ready},
};

use pin_project::pin_project;

use async_coroutine::{AsyncCoroutine, CoroutineState};

/// An asynchronous state machine.
#[pin_project]
pub struct StateMachine<'a, T, F>
where
    F: TransitionFn<T> + 'a,
    T: 'a,
{
    #[pin]
    future: Option<F::Future<'a>>,
    state: Option<T>,
    transition_fn: F,
}

impl<'a, T, F> StateMachine<'a, T, F>
where
    F: TransitionFn<T>,
    T: Clone,
{
    /// Creates a new state machine with the initial state and transition function.
    pub const fn new(init_state: T, transition_fn: F) -> StateMachine<'a, T, F> {
        StateMachine {
            future: None,
            state: Some(init_state),
            transition_fn,
        }
    }

    /// Creates a new state machine by deserializing the initial state and
    /// using the transition function.
    ///
    /// # Errors
    ///
    /// If the deserialization fails, this will return the deserialization error.
    #[cfg(feature = "serde")]
    pub fn deserialize_state<'de, D>(
        deserializer: D,
        transition_fn: F,
    ) -> Result<StateMachine<'a, T, F>, D::Error>
    where
        T: serde::de::Deserialize<'de>,
        D: serde::Deserializer<'de>,
    {
        use serde::de::DeserializeSeed;

        StateMachineSeed::new(transition_fn).deserialize(deserializer)
    }

    /// Returns the latest state of the state machine.
    ///
    /// If (and only if) the state machine has completed, this will return `None`.
    pub fn latest_state(&self) -> Option<&T> {
        self.state.as_ref()
    }
}

impl<'a, T, F> AsyncCoroutine<()> for StateMachine<'a, T, F>
where
    F: TransitionFn<T>,
    T: Clone,
{
    type Yield = T;
    type Return = F::Output;

    fn start_resume(self: Pin<&mut Self>, _arg: ()) {}

    fn poll_resume(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<CoroutineState<T, F::Output>> {
        let mut this = self.project();

        if this.future.is_none() {
            let state = this
                .state
                .clone()
                .expect("state machine resumed after completion");

            // SAFETY: we make sure that `self.future`, which this borrows from,
            // outlives `self.transition_fn`, and that `self.transition_fn` is
            // never accessed until `self.future` is cleared again.
            let transition_fn =
                unsafe { core::mem::transmute::<&mut F, &mut F>(this.transition_fn) };

            let future = transition_fn.transition(state);

            this.future.set(Some(future));
        }

        let future = this.future.as_mut().as_pin_mut().unwrap();

        let result = ready!(future.poll(cx));
        this.future.set(None);

        match result {
            Continue(new_state) => {
                *this.state = Some(new_state.clone());
                Poll::Ready(CoroutineState::Yielded(new_state))
            }
            Break(output) => {
                *this.state = None;
                Poll::Ready(CoroutineState::Complete(output))
            }
        }
    }
}

impl<'a, T, F> Future for StateMachine<'a, T, F>
where
    F: TransitionFn<T>,
    T: Clone,
{
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<F::Output> {
        loop {
            // we skip the call to `start_resume` because we know that for this type, it is a no-op,
            // even if it's required for async coroutines generally.

            if let CoroutineState::Complete(output) = ready!(self.as_mut().poll_resume(cx)) {
                return Poll::Ready(output);
            }
        }
    }
}

#[cfg(feature = "serde")]
impl<T, F> serde::Serialize for StateMachine<'_, T, F>
where
    F: TransitionFn<T>,
    T: serde::Serialize + Clone,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.state.serialize(serializer)
    }
}

/// A trait for transition functions that can be used with `StateMachine`.
///
/// This trait is defined for all types that implement `FnMut(T) -> impl
/// Future<Output = ControlFlow<R, T>>`.
///
/// If the `nightly` feature is enabled, it is implemented for all types that implement
/// `AsyncFnMut(T) -> ControlFlow<R, T>`, which allows for asynchronous closures that
/// capture their environment.
pub trait TransitionFn<T> {
    /// The output type of the transition function.
    type Output;

    /// The future type returned by the transition function.
    type Future<'a>: Future<Output = ControlFlow<Self::Output, T>>
    where
        Self: 'a;

    /// Transition the state machine to a new state.
    fn transition(&mut self, state: T) -> Self::Future<'_>;
}

#[cfg(not(feature = "nightly"))]
impl<F, R, T, Fut> TransitionFn<T> for F
where
    F: FnMut(T) -> Fut,
    Fut: Future<Output = ControlFlow<R, T>>,
{
    type Output = R;
    type Future<'a>
        = Fut
    where
        Self: 'a;

    fn transition(&mut self, state: T) -> Self::Future<'_> {
        self(state)
    }
}

#[cfg(feature = "nightly")]
impl<F, R, T> TransitionFn<T> for F
where
    F: AsyncFnMut(T) -> ControlFlow<R, T>,
{
    type Output = R;

    type Future<'a>
        = TransitionFnFuture<'a, F, R, T>
    where
        Self: 'a;

    #[define_opaque(TransitionFnFuture)]
    fn transition(&mut self, state: T) -> Self::Future<'_> {
        self(state)
    }
}

#[cfg(feature = "nightly")]
#[doc(hidden)]
pub type TransitionFnFuture<'a, F, R, T>
where
    F: AsyncFnMut(T) -> ControlFlow<R, T> + 'a,
= impl Future<Output = ControlFlow<R, T>> + use<'a, F, R, T>;

/// A seed for deserializing a `StateMachine`.
#[cfg(feature = "serde")]
pub struct StateMachineSeed<'a, T, F>
where
    F: TransitionFn<T> + 'a,
    T: 'a,
{
    _lifetime: PhantomData<&'a ()>,
    _arg: PhantomData<fn(T) -> T>,

    transition_fn: F,
}

#[cfg(feature = "serde")]
impl<'a, T, F> StateMachineSeed<'a, T, F>
where
    F: TransitionFn<T>,
{
    /// Creates a new `StateMachineSeed` with the given transition function.
    pub fn new(transition_fn: F) -> Self {
        Self {
            _lifetime: PhantomData,
            _arg: PhantomData,

            transition_fn,
        }
    }
}

#[cfg(feature = "serde")]
impl<'a, 'de, T, F> serde::de::DeserializeSeed<'de> for StateMachineSeed<'a, T, F>
where
    F: TransitionFn<T> + 'a,
    T: serde::de::Deserialize<'de> + 'a,
{
    type Value = StateMachine<'a, T, F>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let state = T::deserialize(deserializer)?;
        Ok(StateMachine {
            future: None,
            state: Some(state),
            transition_fn: self.transition_fn,
        })
    }
}
