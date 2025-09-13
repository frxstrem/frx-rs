#![allow(missing_docs)]
#![cfg_attr(feature = "nightly", feature(coroutine_trait))]

use core::{
    ops::ControlFlow::{self, Break, Continue},
    pin::pin,
    time::Duration,
};

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::time::Instant;

use async_coroutine::{
    AsyncCoroutineExt,
    CoroutineState::{Complete, Yielded},
};
use state_machine::StateMachine;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum State {
    Init,
    Foo(i32),
}

#[tokio::test]
async fn test_state_machine() {
    tokio::time::pause();

    let sm = StateMachine::new(State::Init, state_transition);
    let mut sm = pin!(sm);

    let t = Instant::now();

    assert_eq!(sm.resume(()).await, Yielded(State::Foo(1)));
    assert_eq!(sm.latest_state(), Some(&State::Foo(1)));

    assert_eq!(sm.resume(()).await, Yielded(State::Foo(2)));
    assert_eq!(sm.latest_state(), Some(&State::Foo(2)));
    assert_eq!(t.elapsed(), Duration::from_millis(500));

    assert_eq!(sm.resume(()).await, Yielded(State::Foo(3)));
    assert_eq!(sm.latest_state(), Some(&State::Foo(3)));
    assert_eq!(t.elapsed(), Duration::from_millis(1000));

    assert_eq!(sm.resume(()).await, Yielded(State::Foo(4)));
    assert_eq!(sm.latest_state(), Some(&State::Foo(4)));
    assert_eq!(t.elapsed(), Duration::from_millis(1500));

    assert_eq!(sm.resume(()).await, Yielded(State::Foo(5)));
    assert_eq!(sm.latest_state(), Some(&State::Foo(5)));
    assert_eq!(t.elapsed(), Duration::from_millis(2000));

    assert_eq!(sm.resume(()).await, Yielded(State::Foo(6)));
    assert_eq!(sm.latest_state(), Some(&State::Foo(6)));
    assert_eq!(t.elapsed(), Duration::from_millis(2500));

    assert_eq!(sm.resume(()).await, Complete(6));
    assert_eq!(sm.latest_state(), None);
    assert_eq!(t.elapsed(), Duration::from_millis(2500));
}

#[tokio::test]
async fn test_state_machine_future() {
    tokio::time::pause();

    let sm = StateMachine::new(State::Init, state_transition);
    let mut sm = pin!(sm);

    let t = Instant::now();

    assert_eq!(sm.resume(()).await, Yielded(State::Foo(1)));
    assert_eq!(sm.latest_state(), Some(&State::Foo(1)));

    assert_eq!(sm.resume(()).await, Yielded(State::Foo(2)));
    assert_eq!(sm.latest_state(), Some(&State::Foo(2)));
    assert_eq!(t.elapsed(), Duration::from_millis(500));

    assert_eq!((&mut sm).await, 6);
    assert_eq!(sm.latest_state(), None);
    assert_eq!(t.elapsed(), Duration::from_millis(2500));
}

#[tokio::test]
async fn test_state_machine_stream() {
    tokio::time::pause();

    let sm = StateMachine::new(State::Init, state_transition);

    let states = sm.into_state_stream().collect::<Vec<_>>().await;

    assert_eq!(
        &*states,
        &[
            Yielded(State::Foo(1)),
            Yielded(State::Foo(2)),
            Yielded(State::Foo(3)),
            Yielded(State::Foo(4)),
            Yielded(State::Foo(5)),
            Yielded(State::Foo(6)),
            Complete(6),
        ]
    );
}

#[cfg(feature = "serde")]
#[tokio::test]
async fn test_state_machine_serialization() {
    tokio::time::pause();

    let sm = StateMachine::new(State::Init, state_transition);
    let mut sm = pin!(sm);

    assert_eq!(sm.resume(()).await, Yielded(State::Foo(1)));
    assert_eq!(sm.resume(()).await, Yielded(State::Foo(2)));
    assert_eq!(sm.resume(()).await, Yielded(State::Foo(3)));
    assert_eq!(sm.latest_state(), Some(&State::Foo(3)));

    assert!(
        tokio::time::timeout(Duration::from_millis(100), sm.resume(()))
            .await
            .is_err()
    );
    assert_eq!(sm.latest_state(), Some(&State::Foo(3)));

    // try serializing the state machine
    let state = serde_json::to_value(&*sm).unwrap();
    assert_eq!(state, serde_json::json!({"Foo": 3}));

    let new_sm = StateMachine::deserialize_state(state, state_transition).unwrap();
    let mut new_sm = pin!(new_sm);

    assert_eq!(new_sm.latest_state(), Some(&State::Foo(3)));

    assert_eq!(new_sm.resume(()).await, Yielded(State::Foo(4)));
    assert_eq!(new_sm.latest_state(), Some(&State::Foo(4)));

    assert_eq!(new_sm.resume(()).await, Yielded(State::Foo(5)));
    assert_eq!(new_sm.latest_state(), Some(&State::Foo(5)));

    assert_eq!(new_sm.resume(()).await, Yielded(State::Foo(6)));
    assert_eq!(new_sm.latest_state(), Some(&State::Foo(6)));

    assert_eq!(new_sm.resume(()).await, Complete(6));
    assert_eq!(new_sm.latest_state(), None);
}

async fn state_transition(state: State) -> ControlFlow<i32, State> {
    match state {
        State::Init => Continue(State::Foo(1)),
        State::Foo(x) if x > 5 => Break(x),
        State::Foo(x) => {
            tokio::time::sleep(Duration::from_millis(499)).await;
            Continue(State::Foo(x + 1))
        }
    }
}
