# Simple non-`'static` `dyn Any` with HKTs

So, how can we reïmplement the previous design, but replacing `Put<'lt> & Remove<'lt>` with
`ForLt!`s?

Well, remember that the key intuition we used previously was this `&'lt str = Combine<'lt, Smth>`
"split" (if `Smth` is `: 'static`, then we can erase it with `TypeId`, and we can get our type back):

  - With `Put<'lt> & Remove<'lt>`:

    ```rs ,ignore
    &'lt str = <&'static str as Put<'lt>>::T
    // conceptually:
             = put_trait::Combine<'lt, &'static str>
    ```

  - With `ForLt!`:

    ```rs ,ignore
    &'lt str = <ForLt!(&'_ str) as ForLt>::Of<'lt>;
    // conceptually:
             = hkt::Combine<'lt, ForLt!(&'_ str)>
    ```

      - bonus: the `&'static str = Combine<'lt, ???>` _conundrum_ of the `Put<'lt>` design:

        ```rs ,ignore
        &'static str = <ForLt!(&'static str) as Put<'lt>>::T
        // conceptually:
                     = hkt::Combine<'lt, ForLt!(&'static str)>
        ```
