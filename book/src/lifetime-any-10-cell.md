<style>
    .tooltip {
        visibility: hidden;
    }

    h1:hover .tooltip {
        visibility: visible;
    }
</style>

<span style="text-align: center;">

# <code><a href="https://doc.rust-lang.org/stable/core/cell/struct.Cell.html">セル</a></code>よひざまずけ!<br/><span class="tooltip">On Your Knees, <code><a href="https://doc.rust-lang.org/stable/core/cell/struct.Cell.html">Cell</a></code>!</span>

<a href="https://twitter.com/Garretthanna/status/1299407475188346880"><img
    src = "https://user-images.githubusercontent.com/9920355/260236082-8515719e-5589-4166-94d4-f312514d40a3.png"
    height = "300px"
    title = "Credit to https://twitter.com/Garretthanna/status/1299407475188346880"
    alt = "just a drawing of DBZ Cell character, to set the decor"
/></a>

</span>

Let's now consider, for instance, the type `Cell<&'u i32>`:

```rust ,ignore
/// Is this sound?
pub
fn we_do_a_lil_unsafe_2<'u>(
    r: Cell<&'u i32>,
) -> Box<dyn 'u + MyAny>
{
    // SAFETY:
    //  1. this `'static` is immediately erased to `dyn 'u + …`,
    //     so the resulting entity won't be usable beyond `'u`;
    //  2. the only way to extract this `Cell<&i32>` back from a `dyn MyAny` is
    //     through `downcast_cell_ref`, which also yields a `'u`-bounded Cell<&i32>
    let r: Cell<&'static i32> = unsafe { ::core::mem::transmute(r) };
    Box::new(r)
}

impl<'u> dyn 'u + MyAny {
    fn downcast_cell_ref<'r, T : 'static>(
        self: &'r (dyn 'u + MyAny),
    ) -> Option<&'r Cell<&'u T>>
    {
        self.is::<Cell<&'static T>>().then(unsafe {
            &*(self as *const Self as *const Cell<&T>)
        })
    }
}
```

So, is this sound?

<img
    src="https://user-images.githubusercontent.com/9920355/243356995-0bfdfa9f-e2d8-4520-af76-cfd656724f8b.png"
    alt="well yes but actually no"
    title="well yes but actually no"
    height="200px"
/>

Indeed, as long as we were dealing with `&'u i32` kind of references, the only pitfall with which to
be careful was ending up with a `&'new_u i32` wherein `'new_u` might be _bigger_ than `'u` (lifetime
_extension_ for references is unsound).

  - This is why my talk has, up until this point, been about (upper-)_bounding_ the `'new_u` so as
    to ensure that `'u : 'new_u` always hold.

But with `Cell<&'u i32>`, we have not only that problem (lifetime _extension_ being unsound), but we
also have _another constraint_:

  - lifetime _reduction_ behind a `Cell` is unsound too! (we say that `Cell<_>` is **non-covariant**
    (and since it is not contravariant either, we call this invariance)).

    For those unconvinced, see this:

    ```rust ,edition2018
    use ::core::cell::Cell as Mut; // <- just this should already scream non-covariance for `Cell`.

    fn unsound<'r, 'local>(
      s: &'r Mut<&'static str>,
    ) -> &'r Mut<&'local  str>
    {
        // If `Cell`s were covariant, we'd be able to get this API.
        unsafe { ::core::mem::transmute(s) }
    }

    let mut s: &'static str = "static str";
    {
        let at_mut_static: &Mut<&'static str> = Mut::from_mut(&mut s);
        let local:          &/* 'local */str  = &String::from("...");
        Mut::</* &'local str */>::set(
            unsound(at_mut_static), // : &'_ Mut<&'local str>
            local,                  // : &'local str
        );
    } // <- the `String` is dropped; `local` henceforth dangles.
    let _unrelated: String = "UB!".into();
    println!("{s}");
    ```

    Feel free to click on the <i class="fa fa-play play-button"></i> of that snippet. It features UB,
    since it will read a dangling pointer leading to a Use-After-Free, so the exact behavior is
    technically unpredictable. With that being said, I've written it in a way which made `{s}`,
    when I ran it, read the memory of the `_unrelated: String`, thereby printing `UB!`!

    ___

      - Anecdotical aside (feel free to skip it!)

        In fact, I had to cheat a bit in the `downcast_cell_ref` implementation to avoid running
        into a compile-error because of this very non-covariance!

        <details><summary>Click to see</summary>

        ```rust ,ignore
        impl<'u> dyn 'u + MyAny {
            fn downcast_cell_ref<'r, T : 'static>(
                self: &'r (dyn 'u + MyAny),
            ) -> Option<&'r Cell<&'u T>>
            {
                self.is::<Cell<&'static T>>().then(unsafe {
                    &*(self as *const Self as *const Cell<&T>)
                }) //                                     👆
            }
        }
        ```

        </details>

        Indeed, notice how, in the pointer casts, rather than `Cell<&'static i32>`, `Cell<&i32>` has
        been used instead, so that it can _directly_ be inferred to be a `Cell<&'u i32>`.

        Should that not had been done, and `Cell<&'static i32>` had been used in that pointer cast,
        then we would have been unable to get back a `Cell<&'u i32>` out of it due to the lack of
        covariance.

    ___

This means that now we need `'new_u` to be _exactly_ `'u`: if it ends up being _smaller_ than `'u`,
we'll be able to implement the `fn unsound` above using `we_do_a_lil_unsafe_2()`.

In other words, we need to make sure the returned **`dyn 'u + …` is also, somehow, "lower-bounded"
by `'u` too: `'u` must not be able to shrink!**

And this is a problem, since `dyn 'u + MyAny` is a type covariant in `'u`: `'u` is very much allowed
to shrink!

```rust ,edition2018
trait MyAny /* … */ { /* … */ }

fn shrink</* from */ 'big, /* to */ 'small>(
    b: Box<dyn 'big + MyAny>,
) -> Box<dyn 'small + MyAny>
where
    'big : 'small,
{
    b
}
# fn main() { println!("✅"); }
```

  - You may wonder _why_ that is. The reason for it is that `+ 'duration` expresses the
    `+ UsableWithin<'duration>` property, and if something is `UsableWithin<'some_duration>`,
    then _a fortiori_ it is `UsableWithin<'a_smaller_duration>`; much like when something is
    `Fn()` then _a fortiori_ it is `FnMut()`.

How do we solve this?

Very easily, actually: just add an artificial `<'u>` generic lifetime parameter to our `MyAny`
trait.

Indeed, `Trait<'some_lt>`, does not have a specific meaning like `Trait + 'usability` does (which is
what allowed Rust to be lenient and allow shrinking that `+ 'usability`). Within an arbitrary/opaque
`Trait<'lt>` definition, that `'lt` may play any role, including APIs wherein shrinking `'lt` would
be unsound (like our own very case, obviously, but not only that). This means that Rust is not
allowed to modify that `<'lt>` in any way: it will keep it exactly as it initially appears (we say
that `dyn Trait<'lt>` is _invariant_ in `'lt`).

  - Alas, despite the `<'lt>` parameter on `Trait`, the `+ 'usability` parameter is still needed for
    the type, and `Box<dyn Trait<'lt>>` is sugar not for `Box<dyn Trait<'lt> + 'lt>` but for
    `Box<dyn Trait<'lt> + 'static>`, which is a type that combines the worst of both worlds:
      - the presence of `<'lt>` in the type makes it unable to be used beyond it ("`'lt`-infected");
      - the presence of (the implicit) `+ 'static` makes coërcing a concrete type to it more
        difficult (_e.g._, `&'lt i32 : 'static` does not hold).

    In other words, you'd have to prove that your concrete type is `: 'static` only for that
    property to be immediately thrown out of the window because of `<'lt>`.

    Conclusion: we'll have to "stutter" and talk about `dyn 'lt + Trait<'lt>`.

<!--
  - Unless, we use the following trick: if we define `trait Trait<'lt>` as being bounded by `: 'lt`,
    then the meaning of `Box<dyn Trait<'lt>>` changes from `Box<dyn Trait<'lt> + 'static>` to
    `Box<dyn Trait<'lt> + 'lt>`, which is exactly what we wanted! -->

```rust, edition2018
use ::core::{any::TypeId, cell::Cell};

pub
trait MyAny<'lt> : seal::StaticSealed {
    fn type_id(&self) -> TypeId;
}

impl<T : ?Sized + 'static> MyAny<'_> for T {
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

// Main trick: this, much like `Any`'s own `: 'static`, makes `: 'static` a
// mandatory step to be `MyAny`, but the big difference is that despite
// the requirement, we don't get the reverse implication: as far as Rust is
// concerned, there could exist `T : MyAny` for which `T : 'static` would
// not hold!
mod seal {
    pub trait StaticSealed {}
    impl<T : ?Sized + 'static> StaticSealed for T {}
}

// --------------------------------

// from there, the usual downcasting shenanigans:
impl<'u> dyn 'u + MyAny<'u> {
    pub
    fn is<T : 'static>(&self) -> bool {
        self.type_id() == TypeId::of::<T>()
    }

    pub
    fn downcast_bounded_ref<'r, T : 'static>(
        self: &'r (dyn 'u + MyAny<'u>),
    ) -> Option<&'r &'u T>
    {
        self.is::<&'static T>().then(|| unsafe {
            &*(self as *const Self as *const &'static T) // : &'r &'static T
                                                         // : &'r &'u      T
        })
    }

    fn downcast_cell_ref<'r, T : 'static>(
        self: &'r (dyn 'u + MyAny<'u>),
    ) -> Option<&'r Cell<&'u T>>
    {
        self.is::<Cell<&'static T>>().then(|| unsafe {
            &*(self as *const Self as *const Cell<&T>)
        })
    }
}

/// This is sound! 🥳
pub
fn we_do_a_lil_unsafe_2<'u>(
    r: Cell<&'u i32>,
) -> Box<dyn 'u + MyAny<'u>>
{
    // SAFETY:
    //  1. this `'static` is immediately erased to `dyn 'u + …`,
    //     so the resulting entity won't be usable beyond `'u`;
    //  2. the only way to extract this `Cell<&i32>` back from a `dyn MyAny` is
    //     through `downcast_cell_ref`, which also yields a `'u`-bounded Cell<&i32>
    //  3. Variance (or rather lack thereof): the input `r` is invariant in `'u`,
    //     and the returned `dyn 'u + MyAny<'u>` also is, so it won't be possible
    //     to shrink it.
    let r: Cell<&'static i32> = unsafe { ::core::mem::transmute(r) };
    Box::new(r)
}
```

___

<span style="text-align: center;">

<img
    src="https://user-images.githubusercontent.com/9920355/244114830-19584c98-aabe-42d4-9406-e9649268901b.png"
    height="300px"
/>

</span>
