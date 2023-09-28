# Perfect Derive
[![crates.io](https://img.shields.io/crates/v/perfect-derive.svg)](https://crates.io/crates/perfect-derive)
[![crates.io](https://img.shields.io/crates/l/perfect-derive.svg)](https://github.com/LucentFlux/perfect-derive/blob/main/LICENSE)

Adds derive macros for better bounds on generated Copy, Debug, etc. implementations.

See [this blog post](https://smallcultfollowing.com/babysteps//blog/2022/04/12/implied-bounds-and-perfect-derive/) for a summary of the issue.

Since Rust cannot handle cyclic bounds, these macros won't always work. Ideally, in a few years this crate can become a no-op and there will be some way to do this in plain Rust, but until then this hack helps clean up some code.

## The Issue

Taken from Niko's blog above:

```
#[derive(Clone)]
struct List<T> {
    data: Rc<T>,
    next: Option<Rc<List<T>>>,
}

impl<T> Deref for List<T> {
    type Target = T;

    fn deref(&self) -> &T { &self.data }
}
```

Currently, derive is going to generate an impl that requires `T: Clone`, like this…

```
impl<T> Clone for List<T> 
where
    T: Clone,
{
    fn clone(&self) {
        List {
            value: self.value.clone(),
            next: self.next.clone(),
        }
    }
}
```

The `T: Clone` requirement here is not actually necessary. This is because the only `T` in this struct is inside of an `Rc`, and hence is reference counted. Cloning the `Rc` only increments the reference count, it doesn’t actually create a new `T`.

With *perfect derive*, we can do the following instead:

```rust
#[perfect_derive(Clone)]
struct List<T> { /* as before */ }
```

Which generates "better" bounds on the implementation:

```rust
impl<T> Clone for List<T> 
where
    Rc<T>: Clone, // type of the `value` field
    Option<Rc<List<T>>: Clone, // type of the `next` field
{
    fn clone(&self) { /* as before */ }
}
```

Note that these bounds do not require that `T` is itself clonable.
