# Summary

[Introduction](README.md)

- [What are HKTs](what-are-hkts.md)

- [A motivating example: <code>sort_by_key()</code>](motivating-example.md)
  - [Explanation of the issue](explain-sort-by-lifetimes.md)
  - [Can we be fully generic over the return type?](generic-output-sort-by-lifetimes.md)

# Using HKTs for Fun and Profit™

- [Motivation: lifetime-infected `dyn Any` erasure](lifetime-any.md)
    - [Simple non-`'static` `dyn Any` with HKTs](lifetime-any-hkt.md)

# Some Section

- [Some chapter]()

___


# Niche stuff

- [<code>impl\<#\[may_dangle\] …\> Drop</code>]()

___

- [Appendix]()

    - [Elided lifetimes / what is `'_`]()

    - [`Trait + 'lifetime`]()

    - [Subtyping _vs._ Coercions]()
___

[Closing thoughts]()
