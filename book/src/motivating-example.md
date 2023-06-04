# A motivating example

### Situation

Imagine having:

```rust
struct Client {
    id: String,
    tier: u8,
}
```

and then wanting to _sort_ some `&mut [Client]` slice of `Client`s. The thing is, you don't want to
be implementing `Ord` for `Client`, since there is no canonical/absolute ordering of `Client`s:
  - sometimes you may want to sort them based on their `.id`;
  - and sometimes you may want to sort them based on their `.tier`.

So you cannot directly use `slice::sort()`, but luckily you notice there is a special API for
sorting based on a field of our choosing:

```rust
# struct Client { id: String, tier: u8 }
fn sort_clients_by_tier(clients: &mut [Client]) {
    clients.sort_by_key(|client| client.tier)
}
```

So far, so good, but now say you want to implement the other sorting, the one based on the `.id`:

```rust ,compile_fail
# struct Client { id: String, tier: u8 }
fn sort_clients_by_id(clients: &mut [Client]) {
    clients.sort_by_key(|client| &client.id)
}
```

This fails! With:

```rust ,ignore
ERROR MESSAGE HERE
```

The `.tier` case worked because `u8` was `Copy` so we directly returned it from our closure, but
for `.id`s we have to deal with `String`s, which are not `Copy` nor cheap to `Clone`, so the
following, even if it works, would be silly and is out of the question:

```rust
# struct Client { id: String, tier: u8 }
fn sort_clients_by_id(clients: &mut [Client]) {
    clients.sort_by_key(|client| client.id.clone())
}
```

Consider this **6-year-old** issue:

> [`slice::sort_by_key` has more restrictions than `slice::sort_by`](https://github.com/rust-lang/rust/issues/34162)
