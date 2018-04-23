use std::iter::Peekable;
use std::str::Chars;
use std::u32;

pub trait UrlParser: Sized {
    fn parse_from_url(&mut Peekable<Chars>) -> Option<Self>;
}

macro_rules! impl_num_parser {
    ($num:ty) => {
        impl UrlParser for $num {
            fn parse_from_url(iter : &mut Peekable<Chars>) -> Option<$num> {
                let mut consumed = 0 as $num;
                let mut result = 0 as $num;
                for ch in iter {
                    consumed += 1;
                    let x = match ch.to_digit(10) {
                        Some(x) => x as $num,
                        None => return None,
                    };
                    result = match result.checked_mul(10) {
                        Some(result) => result,
                        None => return None,
                    };
                    result = match result.checked_add(x) {
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
        }
    }
}

impl_num_parser!(u8);
impl_num_parser!(u16);
impl_num_parser!(u32);
impl_num_parser!(u64);
impl_num_parser!(i8);
impl_num_parser!(i16);
impl_num_parser!(i32);
impl_num_parser!(i64);

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
            use UrlParser;
            use matches;

            let mut url_iter = $url.chars().peekable();
            branch!($request_verb, url_iter, $default, $( $body, $verb, ( $( $match_url )+ ) ),+ )
        }
    );
    ($url:expr, $( ( $( $match_url:tt )+ ) => $body:expr ),+, _ => $default:expr) => (
        {
            #[allow(unused_imports)]
            use UrlParser;
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
    ($iter:ident, $body:expr, $first:ident : $ty:ty, $( $rest:tt )+) => (
        if let Some($first) = <$ty as UrlParser>::parse_from_url(&mut $iter) {
            predicates!($iter, $body, $($rest)+)
        } else { None }
    );
    ($iter:ident, $body:expr, $first:ident : $ty:ty) => (
        if let Some($first) = <$ty as UrlParser>::parse_from_url(&mut $iter) {
            Some($body)
        } else { None }
    );
}

#[cfg(test)]
mod tests {
    pub const GET: &'static str = "GET";
    pub const POST: &'static str = "POST";

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
            ("/foo/", _id:u8) => true,
            _ => false
        ));
    }

    #[test]
    fn parser_failure() {
        assert!(route_match!("/foo/0xDEADBEEF",
            ("/foo/", _id:u32) => false,
            _ => true
        ));
    }

    #[test]
    fn multiarm_success() {
        assert!(route_match!("/foo/5",
            ("/foo/bar") => false,
            ("/foo/", _id:u8) => true,
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
}
