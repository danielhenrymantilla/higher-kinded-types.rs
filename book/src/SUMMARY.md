# Table of Contents

[Table of contents](README.md)

- [What are HKTs](what-are-hkts.md)

- [A motivating example: `sort_by_key()`](motivating-example.md)
  - [Explanation of the issue](explain-sort-by-lifetimes.md)
  - [Can we be fully generic over the return type?](generic-output-sort-by-lifetimes.md)
  - [Solving it with HKTs](hkts-sort-by-lifetimes.md)

# Using HKTs for Fun and Profit™

- [Motivation: lifetime-infected `dyn Any` erasure](lifetime-any.md)
    - [Simple non-`'static` `dyn Any` with HKTs](lifetime-any-hkt.md)

# Some Section

- [Some chapter]()

___


# Niche stuff

- [`impl\<#\[may_dangle\] …\> Drop`]()

___

- [Appendix]()

    - [Elided lifetimes / what is `'_`]()

    - [`Trait + 'lifetime`]()

    - [Subtyping _vs._ Coercions]()
___

[Closing thoughts]()
