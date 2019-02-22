//! `silk_router` is a URL routing library inspired by rust's pattern matching
//! syntax.
//!
//! ```rust
//!     use silk_router::route_match;
//! 
//!     route_match!(request.verb, request.url,
//!        GET ("/user") => user_list(),
//!        GET ("/user/", id = num::<u32>) => user_details(id),
//!        POST ("/user") => create_user(),
//!        PUT ("/user/", id = num::<u32>) => update_user(id),
//!        _ => error(404, "Not Found")
//!    );
//! ```
//!
//! It is agnostic to the HTTP library you are using. HTTP verbs are checked for
//! strict equality. The URL must support a `.chars()` method returning a
//! `std::str::Chars`. The verbs can be omitted, especially useful for nesting
//! match statements:
//!
//! ```rust
//!     use silk_router::route_match;
//!
//!     route_match!(request.verb, request.url,
//!         GET ("/user", id = num::<u32>, sub_url = rest) => {
//!             let user = load_user(id);
//!             route_match!(sub_url
//!                 ("/settings") => user_settings(user),
//!                 ("/token") => user_token(user),
//!                 _ => user_info(user)
//!             )
//!         }
//!     );
//! ```
//!
//! Inside the match expressions, strings are checkd for exact equality.
//! Expression matches: `ident = expr` call the function returned by `expr`
//! which must be a `FnMut(&mut Peekable<Chars>) -> Option<T>`. If the branch
//! matches, the identifier will be a block-scoped variable with type `T`.
//! 
//! Matches are exhaustive: the entire URL must have been consumed. The
//! `parsers::rest` parser can be used to consume the end of the string.

extern crate num;

use std::iter::Peekable;
use std::str::Chars;

pub mod parsers;

pub fn matches(iter: &mut Peekable<Chars>, text: &'static str) -> bool {
    for ch in text.chars() {
        let other: char = match iter.peek() {
            Some(&ch) => ch,
            None => return false,
        };
        if other != ch {
            return false;
        }
        iter.next();
    }
    return true;
}

/// Pattern-match-style URL routing
#[macro_export]
macro_rules! route_match {
    ($request_verb:expr, $url:expr, $( $verb:ident ( $( $match_url:tt )+ ) => $body:expr ),+, _ => $default:expr) => (
        {
            let mut url_iter = $url.chars().peekable();
            $crate::branch!($request_verb, url_iter, $default, $( $body, $verb, ( $( $match_url )+ ) ),+ )
        }
    );
    ($url:expr, $( ( $( $match_url:tt )+ ) => $body:expr ),+, _ => $default:expr) => (
        {
            let mut url_iter = $url.chars().peekable();
            $crate::branch!(url_iter, $default, $( $body, ( $( $match_url )+ ) ),+ )
        }
    )
}

#[macro_export]
macro_rules! branch {
    ($request_verb:expr, $iter:ident, $default:expr, $body:expr, $verb:expr, ( $( $url:tt )+ ), $( $bodies:expr, $verbs:ident, ( $( $urlses:tt )+ ) ),+) => {
        {
            let mut next_iter = $iter.clone();
            if let Some(result) = $crate::match_verb!($request_verb, $verb, next_iter, $body, $($url)+) {
                result
            } else {
                $crate::branch!($request_verb, $iter, $default, $( $bodies, $verbs, ( $( $urlses )+ ) ),+ )
            }
        }
    };
    ($request_verb:expr, $iter:ident, $default:expr, $body:expr, $verb:expr, ( $( $url:tt )+ ) ) => {
        if let Some(result) = $crate::match_verb!($request_verb, $verb, $iter, $body, $($url)+) {
            result
        } else {
            $default
        }
    };
    ($iter:ident, $default:expr, $body:expr, ( $( $url:tt )+ ), $( $bodies:expr, ( $( $urlses:tt )+ ) ),+) => {
        {
            let mut next_iter = $iter.clone();
            if let Some(result) = predicates!(next_iter, $body, $($url)+) {
                result
            } else {
                $crate::branch!($iter, $default, $( $bodies, ( $( $urlses )+ ) ),+ )
            }
        }
    };
    ($iter:ident, $default:expr, $body:expr, ( $( $url:tt )+ ) ) => {
        if let Some(result) = $crate::predicates!($iter, $body, $($url)+) {
            result
        } else {
            $default
        }
    };
}

#[macro_export]
macro_rules! match_verb {
    ($request_verb:expr, $verb:expr, $iter:ident, $body:expr, $( $url:tt )+) => (
        if $request_verb == $verb {
            $crate::predicates!($iter, $body, $($url)+)
        } else { None }
    )
}

#[macro_export]
macro_rules! predicates {
    ($iter:ident, $body:expr, $first:tt, $( $rest:tt )+) => (
        if $crate::matches(&mut $iter, $first) {
            $crate::predicates!($iter, $body, $($rest)+)
        } else { None }
    );
    ($iter:ident, $body:expr, $first:tt) => (
        if $crate::matches(&mut $iter, $first) {
            if $iter.peek() == None {
                Some($body)
            } else { None }
        } else { None }
    );
    ($iter:ident, $body:expr, $first:ident = $parser:expr, $( $rest:tt )+) => (
        if let Some($first) = $parser(&mut $iter) {
            $crate::predicates!($iter, $body, $($rest)+)
        } else { None }
    );
    ($iter:ident, $body:expr, $first:ident = $parser:expr) => (
        if let Some($first) = $parser(&mut $iter) {
            if $iter.peek() == None {
                Some($body)
            } else { None }
        } else { None }
    );
}

#[cfg(test)]
mod tests {
    pub const GET: &'static str = "GET";
    pub const POST: &'static str = "POST";

    use super::{route_match};
    use super::parsers::{num, rest, until};

    #[test]
    fn match_success() {
        assert!(route_match!("/foo/bar",
            ("/foo/bar") => true,
            _ => false
        ));
    }

    #[test]
    fn match_failure() {
        assert!(route_match!("/foo/fail",
            ("/foo/bar") => false,
            _ => true
        ));
    }

    #[test]
    fn parser_success() {
        assert!(route_match!("/foo/5",
            ("/foo/", id = num::<u8>) => id == 5,
            _ => false
        ));
    }

    #[test]
    fn parser_failure() {
        assert!(route_match!("/foo/0xDEADBEEF",
            ("/foo/", _id = num::<u32>) => false,
            _ => true
        ));
    }

    #[test]
    fn first_match() {
        assert!(route_match!("/foo",
            ("/foo") => true,
            ("/foo") => false,
            _ => false
        ));
    }

    #[test]
    fn multiarm_success() {
        assert!(route_match!("/foo/5",
            ("/foo/bar") => false,
            ("/foo/", _id = num::<u8>) => true,
            _ => false
        ));
    }

    #[test]
    fn match_verb() {
        assert!(route_match!(POST, "/foo/bar",
            GET ("/foo/bar") => false,
            POST ("/foo/bar") => true,
            _ => false
        ));
    }

    #[test]
    fn match_fun() {
        assert!(route_match!(GET, "/foo/groucho:swordfish",
            GET ("/foo/", username = until(':'), password = rest) => 
                username == "groucho" && password == "swordfish",
            _ => false
        ));
    }

    #[test]
    fn match_full() {
        assert!(route_match!("/abcde",
            ("/abcd") => false,
            ("/abcde") => true,
            _ => false
        ));
    }
}
