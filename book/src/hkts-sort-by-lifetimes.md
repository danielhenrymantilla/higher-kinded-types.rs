# Solving it with HKTs

From the previous section, we had:

> ```rust ,edition2018,compile_fail
> use ::core::cmp::Reverse;
> # struct Client { id: String }
>
> fn intuition<Output /* <'_> */>(
>     _get_key: impl for<'item>
>         FnMut(&'item Client) -> Output<'item>
>     ,
> )
> {}
>
> type E<'item> = Reverse<&'item String>;
>
> intuition::<E>(|c: &'_ Client| -> E<'_> {
>     Reverse(&c.id)
> });
> # println!("✅");
> ```
>
> yielding:
>
> ```rust ,ignore
> error[E0109]: lifetime arguments are not allowed on type parameter `Output`
>  --> src/main.rs:9:40
>   |
> 9 |         FnMut(&'item Client) -> Output<'item>
>   |                                 ------ ^^^^^ lifetime argument not allowed
>   |                                 |
>   |                                 not allowed on type parameter `Output`
>   |
> note: type parameter `Output` defined here
>  --> src/main.rs:6:14
>   |
> 6 | fn intuition<Output /* <'_> */>(
>   |              ^^^^^^
>
> For more information about this error, try `rustc --explain E0109`.
> ```

And we suspected that HKTs and "`For` types" could be quite handy here. Let's see them in action 🚀

### 1. Identify the "nature" / _kind_ / arity of our nested genericity:

> ```rust ,ignore
> fn intuition<Output /* <'_> */>(
> //                      👆
> //                    one lifetime
> //                     parameter
> ```

We got _one lifetime_ parameter, so this is a good fit for [`ForLifetime`]:

  - ```rust ,ignore
    //! From `::higher-kinded-types`

    trait ForLifetime {
        /// Gat with:
        ///
        ///  one lifetime
        ///     👇
        type Of<'__>;
    }
    ```

  - See the `::higher_kinded_types::extra_arities` module for other such traits.

### 2. Write our Higher-Kinded API using it:

Let's use that for our `fn` definition, then, shall we?

```rust ,edition2018
use ::core::cmp::Reverse;
use ::higher_kinded_types::{ForLifetime as Ofᐸᑊ_ᐳ};
# extern crate self as higher_kinded_types; trait ForLifetime { type Of<'__>; }
#
# struct Client { id: String }

//                 1) conceptually: "being <'_>"
//                   👇
fn intuition<Output: Ofᐸᑊ_ᐳ>(
    _get_key: impl for<'item>
        FnMut(&'item Client) -> Output::Of<'item>
//                                    👆👆
//                                  2) feed lifetime
    ,
)
{}
# fn main() { println!("✅"); }
```

### 3. Callers use the provided convenience macro

```rust ,edition2018
# #![feature(unboxed_closures)]
use ::core::cmp::Reverse;
use ::higher_kinded_types::{ForLifetime as ForLt};
# extern crate self as higher_kinded_types; trait ForLifetime { type Of<'__>; }
# macro_rules! ForLt {(<$lt:lifetime> = $T:ty) => ( for<$lt> fn(&$lt ()) -> $T )} impl<F : for<'any> FnOnce<(&'any (), )>> ForLt for F { type Of<'lt> = <F as FnOnce<(&'lt (), )>>::Output; }
#
# struct Client { id: String }

//                 1) conceptually: "being <'_>"
//                   👇
fn intuition<Output: ForLt>(
    _get_key: impl for<'item>
        FnMut(&'item Client) -> Output::Of<'item>
//                                    👆👆
//                                  2) feed lifetime
    ,
)
{}
# fn main() {

// 3) Call-site!
/* 3.1 */
// type Ret          <'item> = Reverse<&'item String> ;
   type Ret = ForLt!(<'item> = Reverse<&'item String>);

/* 3.2 */
intuition::<Ret>(|c: &'_ Client| {
    Reverse(&c.id)
});

println!("✅");
# }
```

### 4) And _voilà_! 😙👌

Profit™

___

## Time to apply it to our real example

```rust ,edition2018
# #![feature(unboxed_closures)]
#![forbid(unsafe_code)]

use ::core::cmp::Reverse;

struct Client { tier: Tier, id: Id }
type Tier = u8;
type Id = String;

/// 0. Import the `For…` trait and the convenience macro.
use ::higher_kinded_types::{ForLifetime as ForLt}; // 👈
# extern crate self as higher_kinded_types; trait ForLifetime { type Of<'__>; }
# macro_rules! ForLt {($T:ty) => ( fn(&()) -> $T )} impl<F : for<'any> FnOnce<(&'any (), )>> ForLt for F { type Of<'lt> = <F as FnOnce<(&'lt (), )>>::Output; }

/// 1. Define a Higher-Kinded API by bounding a generic parameter with it.
fn slice_sort_by_key<Key: ForLt> ( // 👈
    slice: &mut [Client],

    // 2. Feed the lifetimes on it as needed.
    mut get_key: impl for<'it> FnMut(&'it Client) -> Key::Of<'it>, // 👈
)                                                 // ^
where                                             // |
    for<'it> Key::Of<'it> : Ord // ------------------+
{
    slice.sort_by(|a, b| Ord::cmp(
        &get_key(a),
        &get_key(b),
    ))
}
# fn main() {

let clients: &mut [Client] = // …
# &mut [];

// 3. Call-sites turbofish the generic with the convenience macro
slice_sort_by_key::<ForLt!{ (Tier, Reverse<&Id>) }>(clients, |c| ( // 👈
    c.tier,
    Reverse(&c.id),
));
# println!("✅");
# }
```

  - #### Bonus: as an extension method

    Or, with the [extension method pattern](https://docs.rs/extension-traits):

    ```rust ,edition2018
    # #![feature(unboxed_closures)]
    #![forbid(unsafe_code)]

    // Given:
    use ::core::cmp::Reverse;

    struct Client { tier: Tier, id: Id }
    type Tier = u8;
    type Id = String;

    use ::higher_kinded_types::{ForLifetime as ForLt};
    # extern crate self as higher_kinded_types; trait ForLifetime { type Of<'__>; }
    # macro_rules! ForLt {($T:ty) => ( fn(&()) -> $T )} impl<F : for<'any> FnOnce<(&'any (), )>> ForLt for F { type Of<'lt> = <F as FnOnce<(&'lt (), )>>::Output; }

    trait SortByDependentKey {
        fn sort_by_dependent_key<Key : ForLt>(
            self: &mut Self,
            get_key: impl for<'it> FnMut(&'it Client) -> Key::Of<'it>,
        )
        where
            for<'it> Key::Of<'it> : Ord,
        ;
    }

    impl SortByDependentKey for [Client] {
        fn sort_by_dependent_key<Key : ForLt>(
            self: &mut [Client],
        // …
        #     mut get_key: impl for<'it> FnMut(&'it Client) -> Key::Of<'it>,
        # )
        # where
        #     for<'it> Key::Of<'it> : Ord,
        # {
        #     self.sort_by(|a, b| Ord::cmp(
        #         &get_key(a),
        #         &get_key(b),
        #     ))
        # }
    }
    # fn main() {

    let clients: &mut [Client] = // …
    # &mut [];
    clients.sort_by_dependent_key::<ForLt!{ (Tier, Reverse<&Id>) }>(|c| (
        c.tier,
        Reverse(&c.id),
    ));
    # println!("✅");
    # }
    ```
