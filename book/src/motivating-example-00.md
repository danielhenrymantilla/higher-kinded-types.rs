<span style="text-align: center;">

# A motivating example: `sort_by_key()`

</span>

### Situation

Imagine having:

```rust ,edition2018
struct Client {
    tier: u8,
    id: String,
}
```

and then wanting to _sort_ some `&mut [Client]` slice of `Client`s. The thing is, you don't want to
be implementing `Ord` for `Client`, since there is no canonical/absolute ordering of `Client`s:
  - sometimes you may want to sort them based on their `.id`;
  - and sometimes you may want to sort them based on their `.tier`.

So you cannot directly use `slice::sort()`.

But luckily, you notice there is a special API precisely to sort based on a field of our choosing:

```rust ,edition2018
# struct Client { id: String, tier: u8 }
fn sort_clients_by_tier(cs: &mut [Client]) {
    cs.sort_by_key(|c| -> u8 { c.tier })
}
# fn main() { println!("✅"); }
```

So far, so good, but now say you want to implement the other sorting, the one based on the `.id`:

```rust ,compile_fail
# struct Client { id: String, tier: u8 }
fn sort_clients_by_id(cs: &mut [Client]) {
    cs.sort_by_key(|c| -> &String { &c.id })
}
# fn main() { println!("✅"); }
```

This fails! With:

```rust ,ignore
error: lifetime may not live long enough
 --> src/lib.rs:3:33
  |
3 | cs.sort_by_key(|c| -> &String { &c.id })
  |                 -     -         ^^^^^ returning this value requires that `'1` outlive `'2`
  |                 |     |
  |                 |     let's call the lifetime of this reference `'2`
  |                 has type `&'1 Client`
```

What happened?

The `.tier` case worked because `u8` was `Copy` so we directly returned it from our closure, but
for `.id`s we have to deal with `String`s, which are not `Copy` nor cheap to `Clone`, so the
following, even if it works, would be silly and is out of the question:

```rust ,edition2018
# struct Client { id: String, tier: u8 }
fn sort_clients_by_id(clients: &mut [Client]) {
    clients.sort_by_key(|client| client.id.clone())
}
```

This is basically the **6-year-old** issue of `slice::sort_by_key`.

> [`slice::sort_by_key` has more restrictions than `slice::sort_by`](https://github.com/rust-lang/rust/issues/34162)
