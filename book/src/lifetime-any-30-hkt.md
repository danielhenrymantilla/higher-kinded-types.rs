# Simple non-`'static` `dyn Any` with HKTs

So, how can we reïmplement the previous design, but replacing `Put<'r> & Remove<'r>` with
`ForLt!`s?

## `type Combine<'r, X> = …`

Well, remember that the key intuition we used previously was this `&'r str = Combine<'r, Smth>` "split" (indeed, if `Smth` is `: 'static`, then we can erase it as a dynamic `TypeId`, and we can get our type back by virtue of `Combine`-ing it with our (non-erased) `'r`):

### With `Put<'r> & Remove<'r>`

```rs ,ignore
mod put_trait {
    use …::Put;
    pub type Combine<'r, X : Put<'r>> = <X as Put<'r>>::T;
    //                   ^^^^^^^^^^^
    //                   limitation: not all types impl `Put<'r>`
}
```

so that:

```rs ,ignore
put_trait::Combine<'r, &'static str>
    = <&'static str as Put<'r>>::T
    =  &'r str
```

### With `ForLt!`

```rs ,ignore
mod hkt {
    use …::ForLt;

    pub type Combine<'r, X : ForLt> = X::Of<'r>;
    //                   ^^^^^^^^^^^
    //                   limitation: requires `ForLt!` macro to use.
    //                   benefit: can handle *all* the `<'r>`-generic types!!
}
```

so that:

```rs ,ignore
hkt::Combine<'r, ForLt!(&str)>
    = <ForLt!(&'_ str) as ForLt>::Of<'r>
    =         &'r str
```

#### Benefits of this design

Remember the `&'static str = Combine<'r, ???>` _conundrum_ of the `Put<'r>` design? Well, with `Combine<'lt, X : ForLt> = X::Of<'lt>`, we have a **simple solution** to this:

 1. instead of using `X = ForLt!(&'_ str)` (_i.e._, `X = ForLt!(<'r> = &'r str)`),
 1. we can simply use `X = ForLt!(&'static str)` (_i.e._, `X = ForLt!(<'r> = &'static str)`):

```rs ,ignore
hkt::Combine<'r, ForLt!(&'static str)>
      = <ForLt!(<'r> = &'static str) as ForLt>::Of<'r>
      = &'static str // ✅
```

And this solutions is also **scalable**, insofar it is able to perfectly tackle arbitrarily complex types such as `Cow<'r, str>`, `fd::Borrowed<'r>`, _etc._:

```rs ,ignore
/// Find X so that:
/// hkt::Combine<'r, X> = Cow<'r, str>
type X = ForLt!(Cow<'_, str>);
```

```rs ,ignore
/// Find X so that:
/// hkt::Combine<'r, X> = fd::Borrowed<'r>
type X = ForLt!(fd::Borrowed<'_>);
```

## `type T = Combine<'soul, Body>`

In fact, if we consider a type's "body" the `TypeId`-runtime-materializable part of a type, and its `'r`-dependent component its "soul", which cannot be materialized / reïfied within runtime / monomorphized data, then this `Combine` operator represents the act of imbuing a body with a soul; and in the other direction, the act of splitting a type `T` into its `'soul` and `Body` constituents:

```rs ,ignore
/// A soul-splitting operation
&'lt str = Combine<'lt, ForLt!(&'_ str)>;
                // ^^^  ^^^^^^^^^^^^^^^
                // soul      body
```

And indeed, once equipped with such a tool/pattern, we are easily able to systematically perform this `dyn Any`-ification of arbitrary `<'lt>`-dependent types:

 1. ```rust
    # pub type PhantomInvariant<'lt> = PhantomData<fn(&'lt ()) -> &'lt ()>;
    # pub trait ForLt { type Of<'__>; }
    pub
    struct Split<'soul, Body : ForLt> {
        _soul: PhantomInvariant<'soul>,
        /// TODO: autotraits
        body: Body::Of<'static>,
        //             ^^^^^^^
        //             since we have the lifetime info alongside this field,
        //             we don't need to repeat it here,
        //             and can put any dummy lifetime in its stead.
    }

    pub
    fn soul_split<'soul, Body : ForLt>(
        value: Body::Of<'soul>,
    ) -> Split<'soul, Body>
    {
        let _soul: PhantomInvariant::<'soul> = <_>::default();
        let body = unsafe {
            // erase the lifetime away.
            ::core::mem::transmute::<
                Body::Of<'soul>,
                Body::Of<'_>,
            >(
                body
            )
        };
        Split { _soul, body }
    }

    impl<'soul, Body, T> Split<'soul, Body>
    where
        Body : ForLt<Of<'soul> = T>,
    {
        pub
        fn into_inner(self: Split<'soul, Body>)
          -> T
        {
            let reïmbued = unsafe {
                // reïmbue the carcass with its `_soul`.
                ::core::mem::transmute::<
                    Body::Of::<'_>,
                    Body::Of::<'soul>,
                >(
                    body
                )
            };
            reïmbued
        }
    }

    /// We can even have `Deref{,Mut}` and so on!
    impl<'soul, Body, T>
        ::core::ops::Deref
    for
        Split<'soul, Body>
    where
        Body : ForLt<Of<'soul> = T>,
    {
        type Target = Body::Of<'soul>;

        fn deref(
            self: &'_ Split<'soul, Body>,
        ) -> &'_ Body::Of<'soul>
        {
            unsafe {
                ::core::mem::transmute::<
                    &Body::Of<'_>,
                    &Body::Of<'soul>,
                >(
                    &self.body
                )
            }
        }
    }
    //
    // pub
    // fn map<'soul, Body : ForLt, F>(
    //     it: Split<'soul, Body>,
    //     f: F,
    // ) -> Split<'soul, ForLt!(<F as FnOnce<(Body::Of<'_>, )>>::Output)>
    // where
    //     F : for<'soul> FnOnce<(Body::Of<'_>, )>,
    // {
    //     soul_split<ForLt!(<F as FnOnce<(Body::Of<'_>, )>>::Output)>>::new(
    //         f(it.into_inner())
    //     )
    // }
    ```

 1. And once we have a `Split<'soul, Body>`, it's easy to see how the `Any`-fication of the body can take place:

    ```rust
    # pub type PhantomInvariant<'lt> = PhantomData<fn(&'lt ()) -> &'lt ()>;
    # pub trait ForLt { type Of<'__>; }
    pub
    struct Split<'soul, Body : ForLt> {
        _soul: PhantomInvariant<'soul>,
        body: Body::Of<'static>,
    }

    pub
    struct AnyAndSoul<'soul> {
        _soul: PhantomInvariant<'soul>,
        body: Box<dyn Any>, // or with `Send + Sync`.
    }

    impl<'soul> AnyAndSoul<'soul> {
        pub
        fn new<Body : ForLt>(
            value: Body::Of<'soul>
        ) -> AnyAndSoul<'soul>
        where
            Body::Of<'static> : 'static,
        {
            let Split { _soul, body } = Split::<Body>::new(value);
            let body: Box<dyn Any> = Box::new(body);
            AnyAndSoul { _soul, body }
        }

        pub
        fn downcast_ref<Body : ForLt>(
            &self,
        ) -> Option<&Body::Of<'soul>>
        where
            Body::Of<'static> : 'static,
        {
            self.body
                .downcast_ref::<Box::Of<'static>>()
                .map(|body| unsafe {
                    ::core::mem::transmute::<
                        &Body::Of<'_>,
                        &Body::Of<'soul>,
                    >(
                        body
                    )
                })
        }
    }
    ```
