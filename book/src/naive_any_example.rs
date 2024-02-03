// ANCHOR: all
mod lib {
    #
    use ::core::{any::TypeId, cell::Cell};

    // ANCHOR: with-and-without-lifetime
    #
    pub
    trait Put<'lt> {
        type Infected;
    }

    // Examples for non-lifetime-generic types:
    impl Put<'_> for String {
        type Infected = Self;
    }
    impl Put<'_> for i32 {
        type Infected = Self;
    }

    // Case of `&'_ T` (more precisely, `&'_ (impl Put…)`):
    // _e.g._, `<&'static i32 as Put<'lt>>::Infected = &'lt i32`.
    impl<'lt, T : Put<'lt>> Put<'lt> for &'static T {
        type Infected = &'lt T::Infected;
    }

    // Similarly, `&'_ Cell<T>` this time:
    impl<'lt, T : Put<'lt>> Put<'lt> for Cell<&'static T> {
        type Infected = Cell<&'lt T::Infected>;
    }

    /* "Reverse" operation: `Remove<'lt>`! */

    //                  This is not strictly needed, but it is intuitive (if it only has
    //                  that one `'lt`, then it has nothing else preventing it from being
    //                  usable within `'lt`) and more importantly it will be quite
    //                  convenient for the incoming `dyn` traits (we'll automagically
    //                  get an implicit `: 'lt` when `: Remove<'lt>`)
    pub //              vvv
    trait Remove<'lt> : 'lt {
        //                          👇             👇
        type Static : 'static + Put<'lt, Infected = Self>;
        //                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^
        //  This `Put` bound is not _strictly_ required to implement our API,
        //  but is the main trick allowing us not to have to deal
        //  with `unsafe trait`s and complex/subtle preconditions,
        //  since basically the safety requirements, for the downcast
        //  below to be sound, are:
        //    - being *exactly* generic over that lifetime and no others;
        //    - being "injective" (X ≠ Y (modulo lt) must imply X::Static ≠ Y::Static)
        //
        //  Since both of these properties were a bit subtle to properly express,
        //    - the `Put<'lt>` abstraction has been written to express this first point;
        //    - and the `Remove<'lt> … where Put<'lt, … = Self>` guarantees our needed
        //      *bijection*.

        fn type_id_of_static() -> TypeId {
            TypeId::of::<Self::Static>()
        }
    }

    /// _e.g._, `<&'lt i32 as Remove<'lt>>::Static = &'static i32`.
    impl<'lt, T : Remove<'lt>> Remove<'lt> for &'lt T {
        type Static = &'static T::Static;
    }

    impl<'lt, T : Remove<'lt>> Remove<'lt> for Cell<&'lt T> {
        type Static = Cell<&'static T::Static>;
    }

    impl Remove<'_> for String {
        type Static = Self;
    }
    impl Remove<'_> for i32 {
        type Static = Self;
    }
    // ANCHOR_END: with-and-without-lifetime

    // ANCHOR: any
    #
    /// Note, as mentioned above, thanks to `: 'lt` on this trait (from `Remove<'lt>`),
    /// we get `dyn MyAny<'lt> = dyn 'lt + MyAny<'lt>`, thereby avoiding the
    /// "lifetime stutter" 😌
    pub
    trait MyAny<'lt> : sealed::Remove<'lt> {
        fn dyn_type_id_of_static(&self) -> TypeId;
    }

    mod sealed {
        /// `dyn`-safe version of the true `Remove<'lt>`.
        ///
        /// `Remove<'lt>` is not `dyn` safe, but we'd like to act as if we had that
        /// super-bound. So instead, we use a sealed trait with a blanket impl.
        pub trait Remove<'lt> : 'lt {}
    }
    impl<'lt, T : Remove<'lt>> sealed::Remove<'lt> for T {}

    impl<'lt, T : Remove<'lt>> MyAny<'lt> for T {
        fn dyn_type_id_of_static(&self) -> TypeId {
            Self::type_id_of_static()
        }
    }

    impl<'lt> dyn MyAny<'lt> {
        pub
        fn is<U : Remove<'lt>>(&self) -> bool {
            // The reason why this check can be trusted in the downcasts below is a
            // bit more subtle than meets the eye (remember the difference between
            // `downcast_ref` and `downcast_bounded_ref` to recall how we cannot afford
            // being wave-handed here! These patterns are very unsound-prone, and rigor
            // is needed).
            //
            // I have thus renamed the generic as `U`, to already start removing the
            // mental bias wherein we already imagine `U` to be `T`: I am calling `T`
            // the original type-erased type of the value contained within this
            // `dyn MyAny<'lt>`.
            //
            // Technically, we have two cases:
            //   - `T = Foo<'lt>`, and `U = SomethingDifferent`.
            //
            //     In that case, `T::Static = Foo<'static>` and `U::Static` will be
            //     different types, and thus, will have distinct `TypeId`s, making the
            //     following comparison fail.
            //
            //       - (EDIT: "different" here is meant as "non-reciprocally-subtypes",
            //         since for those it is sound to "mix them up" (like `Any` does)).
            //
            //   - `T = Foo<'lt>`, and `U = Foo<'another_lt>`.
            //
            //     In that case, `T::Static` and `U::Static` will match, so the
            //     following comparison will succeed, which could be deemed a false
            //     positive? (if `'another_lt ≠ 'lt`).
            //
            //     But, in fact, this is fine, since remember that famous extra bound
            //     on `type Static`: it had to be `: Put<'lt, Infected = Self>`.
            //
            //     So we have `T = <T::Static as Put<'lt>>::Infected`.
            //     Replacing `T::Static` with `U::Static`, we end up with:
            //     `T = <U::Static as Put<'lt>>::Infected = U`, which means the
            //     downcast is sound!
            self.dyn_type_id_of_static() == U::type_id_of_static()
        }

        pub
        fn downcast_ref<'r, T : Remove<'lt>>(
            self: &'r dyn MyAny<'lt>,
        ) -> Option<&'r T>
        {
            self.is::<T>().then(|| unsafe {
                &*(self as *const dyn MyAny<'lt> as *const T)
            })
        }

        /// `d_own_cast`, am I right?
        pub
        fn downcast_owned<T : Remove<'lt>>(
            self: Box<dyn MyAny<'lt>>,
        ) -> Option<T>
        {
            self.is::<T>().then(|| unsafe {
                *Box::from_raw(Box::into_raw(self) as *mut dyn MyAny<'lt> as *mut T)
            })
        }
    }
    // ANCHOR_END: any

    // ANCHOR: coerce
    #
    pub
    fn coërce<'lt, T : Remove<'lt>>(
        it: T,
    ) -> Box<dyn MyAny<'lt>>
    {
        // Look ma: no unsafe!
        Box::new(it) as _
    }
    // ANCHOR_END: coerce

    // ANCHOR: static
    #
    /// Helper to avoid coherence issues
    pub
    struct Static<T : 'static>(
        pub T,
    );
    impl<T : 'static> Put<'_> for Static<T> {
        type Infected = Self;
    }
    impl<T : 'static> Remove<'_> for Static<T> {
        type Static = Self;
    }
    // ANCHOR_END: static
}
use lib::*;

fn main() {
    use ::core::cell::Cell;
    // ANCHOR: main
    #
    let mut any; // single var to prove they all have the same `dyn`-erased type.

    let x: i32 = 42;
    any = coërce(x);
    assert_eq!(any.is::<i32>(), true); // ✅
    assert_eq!(any.is::<String>(), false);
    assert_eq!(any.is::<&i32>(), false);
    assert_eq!(any.is::<Cell<&i32>>(), false);
    assert_eq!(any.downcast_owned::<i32>(), Some(42));

    let s: String = "42".into();
    any = coërce(s);
    assert_eq!(any.is::<i32>(), false);
    assert_eq!(any.is::<String>(), true); // ✅
    assert_eq!(any.is::<&i32>(), false);
    assert_eq!(any.is::<Cell<&i32>>(), false);
    assert_eq!(any.downcast_owned::<String>().as_deref(), Some("42"));

    let local = 42;
    let r: &'_ i32 = &local;
    any = coërce(r);
    assert_eq!(any.is::<i32>(), false);
    assert_eq!(any.is::<String>(), false);
    assert_eq!(any.is::<&i32>(), true); // ✅
    assert_eq!(any.is::<Cell<&i32>>(), false);
    assert_eq!(any.downcast_owned::<&i32>(), Some(&42));

    let c: Cell<&'_ i32> = Cell::new(&local);
    any = coërce(c.clone());
    assert_eq!(any.is::<i32>(), false);
    assert_eq!(any.is::<String>(), false);
    assert_eq!(any.is::<&i32>(), false);
    assert_eq!(any.is::<Cell<&i32>>(), true); // ✅
    assert_eq!(any.downcast_owned::<Cell<&i32>>().map(|c| c.get()), Some(&42));

    // This one works thanks to covariance: we have a `&'r &'local i32`, which
    // subtypes `&'r &'r i32`, which is compatible with `Box<dyn 'r + MyAny<'r>>`.
    //
    // IOW, the same with `&c` would not work (this is the "single-lifetime"
    // requirement which is paramount for this design to work (although it should
    // be possible to define `Put2<'a, 'b>` and `Remove2<'a, 'b>` to handle it)).
    let nested_r: &&i32 = &r;
    any = coërce(nested_r);
    assert!(matches!(any.downcast_owned::<&&i32>(), Some(42)));

    // Thanks to the `Static` for-coherence-but-also-distinct-TypeId newtype wrapper,
    // these `&'static` references have their `TypeId` properly "tagged", so as to
    // make them distinguishable from the `r: &'local i32 <: &'r i32` above.
    // This is how we are able to retrieve back a full non-`'r`-bounded type
    // after the erasure.
    let mut static_r: &'static i32 = &42;
    any = coërce(Static(static_r));
    // Notice how, despite the `'r`-infected `Any`, we are still capable of
    // extracting a fully `'static` type out of it!
    static_r = any.downcast_owned::<Static<&i32>>().unwrap().0;
    assert_eq!(static_r, &42);
    // ANCHOR_END: main

    /* 👈 Comment out this line so as to uncomment the following ones, and see the errors:
    // This one fails because due to `Cell`'s invariance, we cannot get a single-`'r` lifetime
    // `&'r Cell<&'r i32>`. Instead, if it is to only have one lifetime, Rust tries to go with
    // `&'local Cell<&'local i32>`, which makes it complain that `'r` is too small, _i.e._, that `c`
    // does not live long enough:
    let nested_c: &Cell<&i32> = &c;
    any = coërce(nested_c);
    drop(any); // */

    println!("✅");
    drop(c); // (NLL may make certain things compile, so let's prevent it)
}
// ANCHOR_END: all
