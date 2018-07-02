URL routing in the style of rust's pattern matching.

```rust
    route_match!(request.verb, request.url,
        GET ("/user") => user_list(),
        GET ("/user/", id = num::<u32>) => user_details(id),
        POST ("/user") => create_user(),
        PUT ("/user/", id = num::<u32>) => update_user(id),
        _ => error(404, "Not Found")
    );
```

### Advantages

### Performance

TODO: measure and discuss. It should be good, right? :)
* We only scan the URL once for parsing & matching
* Everything can be allocated on the stack

#### Dependency Injection

There is no need for a dependency injection framework because you can simply
inject the right dependencies into your endpoint blocks:

```rust
    route_match!(request.verb, request.url,
        GET ("/authenticated/", remainder:rest) => {
            match user_store(database()) {
                Some(user) => route_match!(remainder,
                    "/user_details" => user_details(user)
                ),
                None => unauthorized_reponse()
            }
        }
    );
```

The same applies to any other sort of middleware: logging, timing, etc. These
are all trivial to implement and compose with the URL map.

### Disadvantages

#### Reverse routing is not possible

I can't think of a way of achieving this while maintaining the other advantages
and I don't think the trade-off is worth it.
