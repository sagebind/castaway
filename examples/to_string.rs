use std::fmt::Display;
use transmogrify::Transmogrify;

/// Like `std::string::ToString`, but with an optimization when `Self` is
/// already a `String`.
///
/// Since the standard library is allowed to use unstable features,
/// `ToString` already has this optimization using the `specialization`
/// feature, but this isn't something normal crates can do.
pub trait FastToString {
    fn fast_to_string(&self) -> String;
}

// Currently transmogrify only works for static types...
impl<T: Display + 'static> FastToString for T {
    fn fast_to_string(&self) -> String {
        // If `T` is already a string, then take a different code path.
        // After monomorphization, this check will be completely optimized
        // away.
        if let Some(string) = self.transmogrify_ref::<String>() {
            // Don't invoke the std::fmt machinery, just clone the string.
            string.to_owned()
        } else {
            // Make use of `Display` for any other `T`.
            format!("{}", self)
        }
    }
}

fn main() {
    println!("specialized: {}", String::from("hello").fast_to_string());
    println!("default: {}", "hello".fast_to_string());
}
