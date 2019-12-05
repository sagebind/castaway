# Transmogrify

> **trans·mog·ri·fy**
>
> _verb_: `HUMOROUS`
> 1. To change or alter greatly and often with grotesque or humorous effect.
>
> _noun_: `HUMOROUS`
> 1. An experimental crate for zero-cost downcasting for limited compile-time function specialization.

![License](https://img.shields.io/github/license/sagebind/transmogrify)

## Usage

Import the `Transmogrify` trait to make the methods `transmogrify_ref`, `transmogrify_mut`, and `transmogrify_into` available on almost any value.

## What is this?

This is an experimental library that implements zero-cost downcasting of types that works on stable Rust. It began as a thought experiment after I had read [this pull request](https://github.com/hyperium/http/pull/369) and wondered if it would be possible to alter the behavior of a generic function based on a concrete type without using trait objects. I stumbled on the "zero-cost"-ness of my findings by accident while playing around with different implementations and examining the generated assembly of example programs.

While the API is quite similar to [Any] in the standard library, it provides better optimized code, but also has more limited uses. If you need to store one or more `Box<?>` objects implementing some trait with the option of downcasting, you are much better off using [Any].

## How does it work?

The implementation is relatively simple:

- Use [TypeId] to check whether the target type is the same type as `Self`.
- Use various unsafe constant expressions for transmuting `Self` into the target type.

By avoiding any complex operations to perform downcasting, we end up with something that the compiler will accept, but also consists mostly of const expressions that are easily optimized away. Take a look at the following contrived example:

```rust
fn display_len<T: Display + 'static>(value: T) -> usize {
    match value.transmogrify_into::<String>() {
        Ok(string) => string.len(),
        Err(value) => format!("{}", value).len(),
    }
}

assert_eq!(display_len(String::from("hello")), 5);
assert_eq!(display_len(42i32), 2);
```

When the generic function `display_len` is monomorphized, we essentially get separate function definitions per possible type for `T` that is used. If we inline the transmogrify calls for readability, the result might look something like this:

```rust
fn display_len_String(value: String) -> usize {
    match {
        if TypeId::of::<String>() == TypeId::of::<String>() {
            Ok(/* magic */)
        } else {
            Err(value)
        }
    } {
        Ok(string) => string.len(),
        Err(value) => format!("{}", value).len(),
    }
}

fn display_len_i32(value: i32) -> usize {
    match {
        if TypeId::of::<i32>() == TypeId::of::<String>() {
            Ok(/* magic */)
        } else {
            Err(value)
        }
    } {
        Ok(string) => string.len(),
        Err(value) => format!("{}", value).len(),
    }
}
```

Since `TypeId::of` is a const function backed by a compiler intrinsic, the first `if` condition is trivially optimized away into `true` for the first implementation and `false` for the latter. This leads to further optimizations of eliminating the impossible branch for each.

The second optimization is in the `/* magic */` expression. The exact expression varies on which transmogrify function you use, but is more or less just a transmute that does nothing except make the compiler accept the dangerous-looking type cast.

After all optimizations, the code that is actually compiled might look more like this:

```rust
fn display_len_String(value: String) -> usize {
    string.len()
}

fn display_len_i32(value: i32) -> usize {
    format!("{}", value).len()
}
```

For this specific scenario, this is just as good as specialization!

## Safety

The implementation, while incredibly small, is like 90% unsafe code. It seems to me that the safe API is sound as it relies on some of the same guarantees of [Any], but I could be mistaken.

## Examples

```rust
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
```

## License

This project's source code and documentation is licensed under the MIT license. See the [LICENSE](LICENSE) file for details.


[Any]: https://doc.rust-lang.org/stable/std/any/trait.Any.html
[TypeId]: https://doc.rust-lang.org/stable/std/any/struct.TypeId.html
