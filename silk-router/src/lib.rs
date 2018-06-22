extern crate num;

use std::iter::Peekable;
use std::str::Chars;

use num::{CheckedAdd, CheckedMul, Num, NumCast};

pub fn num<T>(iter : &mut Peekable<Chars>) -> Option<T> where T: Num+NumCast+CheckedAdd+CheckedMul {
    let mut consumed = 0u8;
    let mut result = T::zero();
    for ch in iter {
        consumed += 1;
        let x = match ch.to_digit(10) {
            Some(x) => <T as NumCast>::from(x).unwrap(),
            None => return None,
        };
        result = match result.checked_mul(&<T as NumCast>::from(10).unwrap()) {
            Some(result) => result,
            None => return None,
        };
        result = match result.checked_add(&x) {
            Some(result) => result,
            None => return None,
        };
    }
    if consumed == 0 {
        None
    } else {
        Some(result)
    }
}

pub fn until(delim: char) -> impl FnMut(&mut Peekable<Chars>) -> Option<String> {
    move |peekable| {
        let mut result = String::new();
        for ch in peekable {
            if ch == delim {
                return Some(result);
            }
            result.push(ch);
        }
        None
    }
}

pub fn rest(iter: &mut Peekable<Chars>) -> Option<String> {
    Some(iter.collect())
}

pub fn matches(iter : &mut Peekable<Chars>, text: &'static str) -> bool {
    for ch in text.chars() {
        let other : char = match iter.peek() {
            Some(&ch) => ch,
            None => return false
        };
        if other != ch {
            return false;
        }
        iter.next();
    }
    return true;
}

#[macro_export]
macro_rules! route_match {
    ($request_verb:ident, $url:expr, $( $verb:ident ( $( $match_url:tt )+ ) => $body:expr ),+, _ => $default:expr) => (
        {
            #[allow(unused_imports)]
            use matches;

            let mut url_iter = $url.chars().peekable();
            branch!($request_verb, url_iter, $default, $( $body, $verb, ( $( $match_url )+ ) ),+ )
        }
    );
    ($url:expr, $( ( $( $match_url:tt )+ ) => $body:expr ),+, _ => $default:expr) => (
        {
            #[allow(unused_imports)]
            use matches;

            let mut url_iter = $url.chars().peekable();
            branch!(url_iter, $default, $( $body, ( $( $match_url )+ ) ),+ )
        }
    )
}

#[allow(unused_macros)]
macro_rules! branch {
    ($request_verb:ident, $iter:ident, $default:expr, $body:expr, $verb:ident, ( $( $url:tt )+ ), $( $bodies:expr, $verbs:ident, ( $( $urlses:tt )+ ) ),+) => {
        {
            let mut next_iter = $iter.clone();
            if let Some(result) = match_verb!($request_verb, $verb, next_iter, $body, $($url)+) {
                result
            } else {
                branch!($request_verb, $iter, $default, $( $bodies, $verbs, ( $( $urlses )+ ) ),+ )
            }
        }
    };
    ($request_verb:ident, $iter:ident, $default:expr, $body:expr, $verb:ident, ( $( $url:tt )+ ) ) => {
        if let Some(result) = match_verb!($request_verb, $verb, $iter, $body, $($url)+) {
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
                branch!($iter, $default, $( $bodies, ( $( $urlses )+ ) ),+ )
            }
        }
    };
    ($iter:ident, $default:expr, $body:expr, ( $( $url:tt )+ ) ) => {
        if let Some(result) = predicates!($iter, $body, $($url)+) {
            result
        } else {
            $default
        }
    };
}

#[allow(unused_macros)]
macro_rules! match_verb {
    ($request_verb:ident, $verb:ident, $iter:ident, $body:expr, $( $url:tt )+) => (
        if $request_verb == $verb {
            predicates!($iter, $body, $($url)+)
        } else { None }
    )
}

#[allow(unused_macros)]
macro_rules! predicates {
    ($iter:ident, $body:expr, $first:tt, $( $rest:tt )+) => (
        if matches(&mut $iter, $first) {
            predicates!($iter, $body, $($rest)+)
        } else { None }
    );
    ($iter:ident, $body:expr, $first:tt) => (
        if matches(&mut $iter, $first) {
            Some($body)
        } else { None }
    );
    ($iter:ident, $body:expr, $first:ident = $parser:expr, $( $rest:tt )+) => (
        if let Some($first) = $parser(&mut $iter) {
            predicates!($iter, $body, $($rest)+)
        } else { None }
    );
    ($iter:ident, $body:expr, $first:ident = $parser:expr) => (
        if let Some($first) = $parser(&mut $iter) {
            Some($body)
        } else { None }
    );
}

#[cfg(test)]
mod tests {
    pub const GET: &'static str = "GET";
    pub const POST: &'static str = "POST";
 
    use super::num;
    use super::until;
    use super::rest;

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
            // GET (foo = five) => true,
            _ => false
        ));
    }
}
