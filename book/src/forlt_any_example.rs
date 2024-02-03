// ANCHOR: all
#![feature(unboxed_closures)]

mod lib {
    use ::core::{any::TypeId, marker::PhantomData};

    type PhantomInvariant<'lt> = PhantomData<fn(&'lt ()) -> &'lt ()>;

    /// Inlined version of `::higher-kinded-types`, for demo purposes.
    /// Do not try to understand it, and avoid mindlessly copy-pasting it.
    pub trait ForLt { type Of<'__>; }
    macro_rules! __ {
        ( $T:ty $(,)? ) => ( fn(&()) -> $T );
        ( <$lt:lifetime> = $T:ty $(,)? ) => ( for<$lt> fn(&$lt ()) -> $T );
    } pub(crate) use __ as ForLt;
    impl<F: for<'lt> FnOnce<(&'lt (), )>> ForLt for F {
        type Of<'lt> = <F as FnOnce<(&'lt (), )>>::Output;
    }

    // ANCHOR: split1
    // ANCHOR: just-split
    // ANCHOR: just-split-shorter
    #
    pub
    struct Split<'soul, Body : ForLt> {
        _soul: PhantomInvariant<'soul>,

        carcass: Body::Of<'static>,
        //                ^^^^^^^
        //             since we have the lifetime info alongside this field,
        //             we don't need to repeat it here,
        //             and can put any dummy lifetime in its stead.
        //
        //             … or so we may think. There are a couple edge cases that
        //             make this technically unsound, but for now let's forget
        //             about these. Keep reading because we will revisit this
        //             point, and actually extract yet another meaningful
        //             pattern off this observation 💡
    }

    pub
    fn soul_split<'soul, Body : ForLt>(
        value: Body::Of<'soul>,
    ) -> Split<'soul, Body>
    {
        let _soul: PhantomInvariant::<'soul> = <_>::default();
        let carcass = unsafe {
            // erase the lifetime away.
            ::core::mem::transmute::<
                Body::Of<'soul>,
                Body::Of<'_>,
            >(
                value
            )
        };
        Split { _soul, carcass }
    }
    // ANCHOR_END: split1
    // ANCHOR: split2

    impl<'soul, Body : ForLt> Split<'soul, Body> {
        pub
        fn into_inner(self: Split<'soul, Body>)
          -> Body::Of<'soul>
    // ANCHOR_END: just-split-shorter
        {
            let reïmbued_body = unsafe {
                // reïmbue the carcass with its `_soul`.
                ::core::mem::transmute::<
                    Body::Of::<'_>,
                    Body::Of::<'soul>,
                >(
                    self.carcass
                )
            };
            reïmbued_body
        }
    }
    // ANCHOR_END: just-split

    /// We can even have `Deref{,Mut}` and so on!
    impl<'soul, Body : ForLt>
        ::core::ops::Deref
    for
        Split<'soul, Body>
    {
        type Target = Body::Of<'soul>;

        fn deref<'r>(
            self: &'r Split<'soul, Body>,
        ) -> &'r Body::Of<'soul>
        {
            unsafe {
                ::core::mem::transmute::<
                    &'r Body::Of<'_>,
                    &'r Body::Of<'soul>,
                >(
                    &self.carcass
                )
            }
        }
    }
    // ANCHOR_END: split2

    // ANCHOR: split-any-body

    /// Sealed trait over the `Split<'soul, Body : 'static>` types exclusively.
    mod seal {
        use super::*;
        pub trait IsSplit {}
        impl<Body : 'static + ForLt> IsSplit for Split<'_, Body> {}
        // Note: `ForLt : 'static` implies that this "higher-order" type, such as:
        //  1. `ForLt!(&str) = ForLt!(&'_ str) = ForLt!(<'soul> = &'soul str)`
        //  2. `ForLt!(&'static str) = ForLt!(<'soul> = &'static str)`
        //  3. `ForLt!(&'x str) = ForLt!(<'soul> = &'x str))`
        // does not itself capture any "outer" lifetime `'x` unless `'x : 'static`.
        // In other words, it rules out types such as `3.`, while keeping `1.`
        // and `2.` around.
    }

    /// A `dyn`-safe `Split<'soul, …>` (with the `…` part erased), which we'll
    /// make downcastable back to its corresponding `Split<…>` implementor.
    pub
    trait SplitAnyBody<'soul> : seal::IsSplit
        //            ^^^^^^^
        //            keep `'soul` part of the `dyn`, *invariantly*.
    where
        Self : 'soul,
        //     ^^^^^
        // for convenience / to avoid the `dyn 'soul + SplitAnyBody<'soul>` stutter.
    {
        /// Needed to check the downcasting (much like `Any` does).
        fn type_id_of_body(&self)
          -> TypeId
        ;
    }

    impl<'soul, Body : 'static + ForLt>
        SplitAnyBody<'soul>
    for
        Split<'soul, Body>
    {
        fn type_id_of_body(&self)
          -> TypeId
        {
            // Note: if it helps, we could replace `TypeId::of::<Body>()` here and
            // below with `TypeId::of::< Split<'_, Body> >()` (where `'_` would be
            // `'static`).
            //
            // In other words, this is equivalent to (bijective with)
            // `TypeId::of::<Self>()`, but for having "lost" the `'soul` in the
            // output (which is why the trait itself keeps the `'soul` around).
            TypeId::of::<Body>()
        }
    }

    impl<'soul> dyn SplitAnyBody<'soul> {
        pub
        fn coërce<Body : 'static + ForLt>(
            value: Body::Of<'soul>,
        ) -> Box<dyn SplitAnyBody<'soul>>
        {
            Box::new(soul_split::<Body>(value)) as _ // `dyn` coërcion/erasure!
        }

        pub
        fn downcast_ref<'r, Body : 'static + ForLt>(
            self: &'r dyn SplitAnyBody<'soul>,
        ) -> Option<&'r Body::Of<'soul>>
        {
            (self.type_id_of_body() == TypeId::of::<Body>())
                .then(|| -> &'r Split<'soul, Body> { unsafe {
                    // SAFETY:
                    //  1. thanks to the `seal`, we know the underlying instance
                    //     behind this `dyn` is necessarily some `Split<'soul, X>`
                    //     (`'soul` being the same since it is invariant everywhere).
                    //  2. `.type_id_of_body()` gives us `TypeId::of::<X>()`.
                    //  3. Since `X : 'static, Body : 'static`, their equal type ids
                    //     imply equal types (the very design underpinning
                    //     `::core::any::Any`!): `X = Body`.
                    &*(
                        self as *const dyn SplitAnyBody<'soul>
                             as *const Split<'soul, Body>
                    )
                }})
                // convenience step:
                .map(|split: &'r Split<'soul, Body>| -> &'r Body::Of<'soul> {
                    &**split
                })
        }
    }
    // ANCHOR_END: split-any-body
}
pub use lib::*;

fn main() {
    // ANCHOR: main
    #
    let local = String::from("local");
    // 1. From a heterogeneous tuple…
    let (a, b, c): (i32, &'_ str, &'static str) = (42, &local, "static");

    // Let's call `'soul = '_` the lifetime of the `&local` borrow.
    // Thinking about the `split` / `Combine` abstraction, if we remove this
    // `'_` from their types, we are left with the following type constructors
    // i.e. `ForLt` types (which are all `: 'static`):
    type A = ForLt!(i32);          // ~ type A<'soul> = i32;
    type B = ForLt!(&'_ str);      // ~ type B<'soul> = &'soul str;
    type C = ForLt!(&'static str); // ~ type C<'soul> = &'static str;


    // 2. …to a homogenous array…
    let anys: [Box<dyn SplitAnyBody<'_>>; 3] = [
        <dyn SplitAnyBody<'_>>::coërce::<A>(a),
        <dyn SplitAnyBody<'_>>::coërce::<B>(b),
        <dyn SplitAnyBody<'_>>::coërce::<C>(c),
    ];
    // 3. …and back!
    let [a, b, c] = &anys;
    let (a, b, c): (i32, &'_ str, &'static str) = (
        *a.downcast_ref::<A>().unwrap(),
        *b.downcast_ref::<B>().unwrap(),
        *c.downcast_ref::<C>().unwrap(),
    );
    dbg!(a, b, c);

    // (notice how the usage of `ForLt` lets `&'local str` and `&'static str`
    // *soundly* coëxist together, in a lossless fashion!)
    drop(anys);
    drop(local);
    dbg!(c); // trying to `dbg!(b)` would fail.
    // ANCHOR_END: main
}
// ANCHOR_END: all
