# Silk

Silk is a collection of libraries for building web services in rust.

It is not ready for you yet. If you need something today, you might want to
look at [Rocket](https://rocket.rs/) or [actix-web](https://actix.rs/book/actix-web/),
they seem pretty good.

Goals for all modules are:

### Silk is a library, not a framework

Libraries don't steal your control flow. A lot of rust's efficiency is gained
from controlling your memory layout and allocating on the stack as much
as possible. Silk tries not to prevent you from doing that.

### Be as ergonomic as possible without sacrificing efficiency

Optimize for large production systems with large amounts of traffic.

### Support asynchronous programming

Silk will try not to impose async programming on you, which not everybody
needs, but it will actively try to accomodate it.

### Build on stable rust

Patches that require nightly won't be considered. Feel free to fork.

## silk-router

URL routing in the style of rust's pattern matching.

```rust
    route_match!(POST, "/foo/bar",
        GET ("/user", ) => user_list(),
        GET ("/user/", id:u32) => user_details(id),
        POST ("/user") => create_user(),
        PUT ("/user/", id:u32) => update_user(id),
        _ => error(404, "Not Found")
    );
```
