/// Construct a `match`-like expression that matches on a value's type.
///
/// # Examples
///
/// ```
/// use std::fmt::Display;
/// use transmogrify::match_type;
///
/// fn to_string<T: Display + 'static>(value: T) -> String {
///     match_type!(value, {
///         String as s => s,
///         &str as s => s.to_string(),
///         default s => s.to_string(),
///     })
/// }
///
/// println!("{}", to_string("foo"));
/// ```
#[macro_export]
macro_rules! match_type {
    ($value:expr, {
        $T:ty as $pat:pat => $branch:expr,
        default $default_pat:pat => $default_branch:expr $(,)*
    }) => {{
        match $crate::Transmogrify::transmogrify_into::<$T>($value) {
            Ok($pat) => $branch,
            Err($default_pat) => $default_branch,
        }
    }};

    ($value:expr, {
        $T:ty as $pat:pat => $branch:expr,
        $($tail:tt)*
    }) => {{
        $crate::match_type!($value, {
            $T as $pat => $branch,
            default value => $crate::match_type!(value, {
                $($tail)*
            })
        })
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn match_type() {
        let v = 42i32;

        assert!(match_type!(v, {
            u32 as _ => false,
            i32 as _ => true,
            default _ => false,
        }));
    }
}

