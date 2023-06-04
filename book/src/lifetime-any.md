# Lifetime-infected `dyn Any` erasure

### A brief introduction about `Any : 'static`

Depending on your Rust needs, you may have needed to use type erasure, using
`dyn Trait`s, and in the more extreme case, you might not have had a single
`Trait` behind which to hide your types other than `Any`, the API that consists
of `dyn`amic/at-runtime type identification, which thereby allows guessing the
originally erased type, so as to undo the `dyn` erasure if we get it right
(called type downcasting).

The issue in question, here, is with lifetimes:

  - `dyn Any` is a type with no lifetime within it (it is `: 'static`), so any lifetime information
    present in the original type before erasure is not present in the static/compile-time/type-level
    information of `dyn Any`.

  - but lifetimes are purely a compile-time construct!

      - (which enables `borrowck`; but technically you can implement a Rust compiler that skips
        borrow-checking and directly tries to compile the (hopefully correct) code into machine
        code, and if you do that then your compiler will be able to fully[^ignoring_lifetimes]
        ignore lifetimes).

[^ignoring_lifetimes]: `for<'a, 'b>`-arity, on the other hand, does play a role and is able to lead to different monomorphizations, though. Otherwise `fn(&str) -> &str` and `fn(&str) -> &'static str)`, which are both `: 'static` would be mixed up by `Any`.

  - this means that it won't be possible to query back that lost lifetime information within the
    runtime/`dyn`amic self-type-identification machinery.

So, because of this, types are not allowed to carry lifetime information/restrictions within them, in
order for them be soundly erasable to `dyn Any`: this property is achieved thanks to the notorious
`: 'static` bound on `Any`.

But this, then, leads to the following legitimate question:

### Is it possible to have a type-erasure mechanism allowing downcasts to lifetime-infected types?

Based on the previous section you may be tempted to say no, except for one important detail around
which we can try to work:

> `dyn Any` is a type with no lifetime within it (it is `: 'static`)

What about `dyn Any + 'lt`, then?

As a thought experiment, let's imagine having a `&'static i32`, such as `&42` (thanks to `static`
promotion LINKME).

This is a `: 'static` type, so it can be coërced to a `dyn Any`, with the `TypeId::of::<&i32>()`.

Now say you have a `&'r i32`. What if, with a lil unsafe, we did the following:

```rust ,ignore
use ::core::any::Any;

/// Is this sound?
fn we_do_a_lil_unsafe<'r>(
    r: &'r i32,
) -> Box<dyn 'r + Any>
{
    // Safety: this `'static` is immediately erased to `dyn 'r + …`, so the
    // resulting entity won't be usable beyond `'r`.
    let r: &'static i32 = unsafe { ::core::mem::transmute(r) };
    Box::new(r)
}
```

What do you think of this snippet? Is it sound?

Well, the following compiling just fine is a bit problematic, is it not?

```rust ,edition2018
use ::core::any::Any;

fn wOnT_bE_uSaBlE_bEyOnD_r(
    r: Box<dyn 'r + Any>
) -> Box<dyn 'static + Any>
{
    r
}
```

  - Indeed, `: 'static` is a super-bound of `Any`, meaning that whenever something is `: Any`,
    then it also is `: 'static`! So a `Box<dyn 'r + Any>` is actually a
    `Box<dyn 'r + 'static + Any>`, which, in turn, is a `Box<dyn 'static + Any>` since `: 'a + 'b`
    is equivalent to `: union('a, 'b) ~ max('a, 'b) = 'static when 'b = 'static` (people often
    trip up on this, since they're so used to seeing `+ 'r` as a max-bound of usability of items,
    when it's actually _a lower bound_. Same as with `FnMut()` and `FnOnce()`, for instance: whilst
    an arbitrary `F : FnOnce()` may only be callable once (conservative assumption _barring extra
    information_), a `F : FnOnce() + FnMut()` is not an oxymoron, but just a plain `F : FnMut()`.
    It's the same with `: 'region_of_usability`)

    In other words, the very `: 'static` slapped onto `Any` for soundness is the one giving a capability
here that makes `dyn Any`s "too strong", and thus, unsound-prone, for custom shenanigans such as
this one at least.

    IT'S IRONIC

Hum, this will require some extra hoops then. Let's try to rewrite `Any` but without the mandatory
`: 'static` available to `dyn`s:

```rust ,edition2018
use ::core::any::TypeId;

// for reference, `Any`'s definition:
#[cfg(feature = "if I were a core 🎶")]
pub
trait Any : 'static {
    fn type_id(&self) -> TypeId {
        impl<T : ?Sized + 'static> Any for T {}

        TypeId::of::<Self>()
    }
}

pub
trait UnboundedAny : seal::StaticSealed {
    fn type_id(&self) -> TypeId;
}

// Main trick: this, much like `Any`'s own `: 'static`, makes `: 'static` a
// mandatory step to be `UnboundedAny`, but the big difference is that despite
// the requirement, we don't get the reverse implicitation: as far as Rust is
// concerned, there could exist `T : UnboundedAny` for which `T : 'static` would
// not hold!
mod seal {
    pub trait StaticSealed {}
    impl<T : ?Sized + 'static> StaticSealed for T {}
}

impl<T : ?Sized + 'static> UnboundedAny for T {
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

// --------------------------------

// from there, the usual downcasting shenanigans:
impl dyn '_ + UnboundedAny {
    pub
    fn is<T : 'static>(&self) -> bool {
        self.type_id() == TypeId::of::<T>()
    }

    pub
    fn downcast_ref<T : 'static>(&self) -> Option<&T> {
        self.is::<T>().then(|| unsafe {
            &*(self as *const Self as *const T)
        })
    }
}

/// Is this sound?
fn we_do_a_lil_unsafe<'r>(
    r: &'r i32,
) -> Box<dyn 'r + UnboundedAny>
{
    // Safety: this `'static` is immediately erased to `dyn 'r + …`,
    // so the resulting entity won't be usable beyond `'r`.
    let r: &'static i32 = unsafe { ::core::mem::transmute(r) };
    Box::new(r)
}
```

And now to check that the `'r` is an effective bound:

```rust ,compile_fail
fn lets_see_the_region_of_usability<'r, 'tell_me>(
    input: Box<dyn 'r + UnboundedAny>
) -> impl 'tell_me + Sized
{
    input
}
```

yields:

```rust ,ignore
error: lifetime may not live long enough
  --> src/lib.rs:67:5
   |
63 | fn lets_see_the_region_of_usability<'r, 'tell_me>(
   |                                     --  -------- lifetime `'tell_me` defined here
   |                                     |
   |                                     lifetime `'r` defined here
...
67 |     input
   |     ^^^^^ function was supposed to return data with lifetime `'tell_me` but it is returning data with lifetime `'r`
   |
   = help: consider adding the following bound: `'r: 'tell_me`
```

So any `'tell_me` wherein a `Box<dyn 'r + UnboundedAny>` may be used has to satisfy `'r ⊆ 'tell_me`,
_i.e._, the biggest such one is `'r` itself, _i.e._, a `Box<dyn 'r + UnboundedAny>` is very much
_not usable beyond `'r`_.

So all is good, right?

![right?](https://user-images.githubusercontent.com/9920355/243207321-63ad631e-8fb6-458e-8aa8-6e44f868386d.png)

```rust ,edition2018
# use ::core::any::TypeId;
#
# pub
# trait UnboundedAny : seal::StaticSealed {
#     fn type_id(&self) -> TypeId;
# }
#
# // Main trick: this, much like `Any`'s own `: 'static`, makes `: 'static` a
# // mandatory step to be `UnboundedAny`, but the big difference is that despite
# // the requirement, we don't get the reverse implicitation: as far as Rust is
# // concerned, there could exist `T : UnboundedAny` for which `T : 'static` would
# // not hold!
# mod seal {
#     pub trait StaticSealed {}
#     impl<T : ?Sized + 'static> StaticSealed for T {}
# }
#
# impl<T : ?Sized + 'static> UnboundedAny for T {
#     fn type_id(&self) -> TypeId {
#         TypeId::of::<Self>()
#     }
# }
#
# // --------------------------------
#
# // from there, the usual downcasting shenanigans:
# impl dyn '_ + UnboundedAny {
#     pub
#     fn is<T : 'static>(&self) -> bool {
#         self.type_id() == TypeId::of::<T>()
#     }
#
#     pub
#     fn downcast_ref<T : 'static>(&self) -> Option<&T> {
#         self.is::<T>().then(|| unsafe {
#             &*(self as *const Self as *const T)
#         })
#     }
# }
#
# /// Is this sound?
# fn we_do_a_lil_unsafe<'r>(
#     r: &'r i32,
# ) -> Box<dyn 'r + UnboundedAny>
# {
#     // Safety: this `'static` is immediately erased to `dyn 'r + …`,
#     // so the resulting entity won't be usable beyond `'r`.
#     let r: &'static i32 = unsafe { ::core::mem::transmute(r) };
#     Box::new(r)
# }
#
fn uh_oh<'r>(
    r: Box<dyn 'r + UnboundedAny>,
) -> &'static i32
{
    *r.downcast_ref::<&i32>().unwrap()
}
```

Indeed, while returning a properly `'r`-bounded entity was _necessary_ for soundness, it was not
_sufficient_: a `'r`-bounded entity may still allow certain APIs to extract non-`'r`-bounded stuff
out of it!

And in this instance, the very API allowing downcasts was the culprit:

```rust ,ignore
impl<'r> dyn 'r + UnboundedAny {
    pub
    fn downcast_ref<T : 'static>(&self) -> Option<&T> {
        (self.type_id() == TypeId::of::<T>()).then(|| unsafe {
            &*(self as *const Self as *const T)
        })
    }
}
```

which, for `T = &'static i32`, becomes:

```rust ,ignore
impl<'r> dyn 'r + UnboundedAny {
    pub
    fn downcast_ref<&'static i32>(&self) -> Option<&&'static i32> {
        (self.type_id() == TypeId::of::<&i32>()).then(|| unsafe {
            &*(self as *const Self as *const T)
        })
    }
}
```

Notice how we end up with a check from `self.type_id()` (which returns `TypeId::of::<&i32>()` for
our constructed value), against `TypeId::of::<&i32>()`[^static].

The check passes, and we end up with a `&'_ &'static i32`, with `'_` being the lifetime of the
`&self = &'_ self = self: &'_ (dyn 'r + UnboundedAny)` receiver.

Which is properly `'_`-bounded and thus `'r`-bounded (with `'_`, itself, `'r`-bounded: `'r ⊆ '_`),
but from which we can simply extract the `&'static i32`, unbounded, by simple `*`-dereference.

Uh-oh.

> So our API is not sound _yet_.

Before tackling a more general fix to this problem, let's palliate it by replacing our `downcast_ref`
above with the following more limited API:

```rust ,ignore
impl<'r> dyn 'r + UnboundedAny {
    /* no more downcast_ref! */
    pub
    fn downcast_bounded_ref<T : 'static>(&self) -> Option<&'r T> {
        (self.type_id() == TypeId::of::<&T>()).then(|| unsafe {
            &*(self as *const Self as *const &T) // : &'_ &'static T
                                                   as &'r T /* deref coërcion */
        })
    }
}
```

[^static]: we are using `&i32` as a shorthand for `&'static i32`, here.
