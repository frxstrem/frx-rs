//! Guard that runs a piece of code once another block of code finishes.
//!
//! See [`guard!`].

#![cfg_attr(not(test), no_std)]

/// Guard that runs a piece of code once another block of code finishes.
///
/// Called as `guard!(<expr>, 'finally: { <finally> })`.
///
/// If `<expr>` returns normally, then `<finally>` is guaranteed to run before
/// the entire `guard!` macro call returns.
///
/// If `<expr>` panics, then `<finally>` is guaranteed to run before the stack
/// unwinds out of the `guard!` macro call.
///
/// Inside of `<finally>` the `is_unwinding!()` macro evalutes to `true` or `false`
/// depending on whether the stack is unwinding due to a panic (`true`) or
/// `<expr>` returned normally (`false`).
#[macro_export]
macro_rules! guard {
    ( $expr:expr ) => {
        $expr
    };

    (
        $expr:expr,
        'finally: $finally:block $(,)?
    ) => {{
        let mut __guard = $crate::drop_guard(|__drop_info| {
            #[allow(unused_macros)]
            macro_rules! is_unwinding {
                () => {
                    __drop_info.unwinding
                };
            }

            $finally
        });

        #[allow(clippy::redundant_closure_call)]
        let __result = (|| $expr)();

        __guard.drop_info.unwinding = false;
        __result
    }};
}

#[doc(hidden)]
pub const fn drop_guard<F: FnOnce(DropInfo)>(f: F) -> DropGuard<F> {
    DropGuard {
        drop_info: DropInfo { unwinding: true },
        f: Some(f),
    }
}

#[doc(hidden)]
pub struct DropGuard<F: FnOnce(DropInfo)> {
    pub drop_info: DropInfo,
    f: Option<F>,
}

impl<F: FnOnce(DropInfo)> Drop for DropGuard<F> {
    fn drop(&mut self) {
        if let Some(f) = self.f.take() {
            f(self.drop_info)
        }
    }
}

#[doc(hidden)]
#[derive(Copy, Clone)]
pub struct DropInfo {
    pub unwinding: bool,
}

#[cfg(test)]
mod tests {
    use core::panic::AssertUnwindSafe;
    use std::{panic::catch_unwind, sync::Mutex};

    use super::*;

    #[derive(Default)]
    struct TestOutput(Mutex<Vec<&'static str>>);

    impl TestOutput {
        pub fn push(&self, s: &'static str) {
            self.0.lock().unwrap().push(s);
        }

        pub fn into_inner(self) -> Vec<&'static str> {
            self.0.into_inner().unwrap()
        }
    }

    #[test]
    fn test_guard() {
        let output = TestOutput::default();

        guard!(output.push("1"), 'finally: {
            if is_unwinding!() {
                output.push("2")
            };

            output.push("3")
        });

        assert_eq!(output.into_inner(), ["1", "3"]);
    }

    #[test]
    fn test_guard_panic() {
        let output = TestOutput::default();

        catch_unwind(AssertUnwindSafe(|| {
            guard!(
                {
                    output.push("1");
                    panic!("some panic")
                },
                'finally: {
                    if is_unwinding!() {
                        output.push("2")
                    }
                    output.push("3")
                }
            )
        }))
        .unwrap_err();

        assert_eq!(output.into_inner(), ["1", "2", "3"]);
    }
}
