/// Provide the value for the first matching type.
///
/// As this branches based on the type needed to be satisfied, only the associated
/// expression will be evaluated. This means that it can be used to
/// return values with conflicting lifetimes, such as a `&mut` or `&` to the same
/// value.
///
/// # Usage
///
/// ```ignore
/// provide_first!(request,
///     Type1 => expr1,
///     Type2 => expr2,
///     Type3 => expr3,
///     // ...
/// );
/// ```
///
/// Returning either a `&mut` or `&` to the same value:
///
/// ```ignore
/// provide_first!(request,
///     &mut T => &mut self.field,
///     &T => &self.field,
/// );
/// ```
///
#[macro_export]
macro_rules! provide_match {
    ($request:expr,
        $( $type:ty => $expr:expr ),+ $(,)?
    ) => {
        {
            let __request: &mut $crate::Request = $request;

            $(
                if __request.would_be_satisfied_by::<$type>() {
                    __request.provide::<$type>($expr)
                } else
            )*
            {
                __request
            }
        }
    };
}
