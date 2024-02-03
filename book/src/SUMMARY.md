# Table of Contents

[Table of contents](README.md)

- [What are HKTs](what-are-hkts.md)

- [A motivating example: `sort_by_key()`](motivating-example-00.md)
  - [Explanation of the issue](motivating-example-10-explain.md)
  - [Can we be fully generic over the return type?](motivating-example-20-genericicty.md)
  - [Solving it with HKTs](motivating-example-30-hkts.md)

# Using HKTs for Fun and Profit™

- [Lifetime-infected `dyn Any` erasure](lifetime-any-00.md)
  - [On Your Knees, <code>Cell</code>!](lifetime-any-10-cell.md)
  - [Fully generalizing this pattern](lifetime-any-20-generalizing.md)
  - [Simple non-`'static` `dyn Any` with HKTs](lifetime-any-30-hkt.md)

# Going further

- [`Unsoundness of `carcass: Body::Of<'static>`](may-dangle-00.md)
  - [Exploiting the unsoundness, method 1](may-dangle-10-oibits.md)
  - [Exploiting the unsoundness, method 2](may-dangle-20-drop.md)

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
