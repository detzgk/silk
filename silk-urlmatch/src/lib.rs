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
macro_rules! urlmatch {
    ($iter:expr, $( $verb:ident ( $( $url:tt )+ ) => $body:expr ),+, _ => $default:expr) => (
        {
            #[allow(unused_imports)]
            use UrlParser;
            use matches;

            let mut url_iter = $iter.chars().peekable();
            branch!(url_iter, $default, $( $body, $verb, ( $( $url )+ ) ),+ )
        }
    )
}

#[allow(unused_macros)]
macro_rules! branch {
    ($iter:ident, $default:expr, $body:expr, $verb:ident, ( $( $url:tt )+ ), $( $bodies:expr, $verbs:ident, ( $( $urlses:tt )+ ) ),+) => {
        {
            let mut next_iter = $iter.clone();
            if let Some(result) = predicates!(next_iter, $body, $($url)+) {
                result
            } else {
                branch!($iter, $default, $( $bodies, $verbs, ( $( $urlses )+ ) ),+ )
            }
        }
    };
    ($iter:ident, $default:expr, $body:expr, $verb:ident, ( $( $url:tt )+ ) ) => {
        if let Some(result) = predicates!($iter, $body, $($url)+) {
            result
        } else {
            $default
        }
    };
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

pub const GET: &'static str = "GET";
pub const POST: &'static str = "POST";

#[cfg(test)]
mod tests {
    #[test]
    fn match_success() {
        assert!(urlmatch!("/foo/bar",
            GET ("/foo/bar") => true,
            _ => false
        ));
    }

    #[test]
    fn match_failure() {
        assert!(urlmatch!("/foo/fail",
            GET ("/foo/bar") => false,
            _ => true
        ));
    }

    #[test]
    fn parser_success() {
        assert!(urlmatch!("/foo/5",
            GET ("/foo/", _id:u8) => true,
            _ => false
        ));
    }

    #[test]
    fn parser_failure() {
        assert!(urlmatch!("/foo/0xDEADBEEF",
            GET ("/foo/", _id:u32) => false,
            _ => true
        ));
    }

    #[test]
    fn multiarm_success() {
        assert!(urlmatch!("/foo/5",
            GET ("/foo/bar") => false,
            GET ("/foo/", _id:u8) => true,
            _ => false
        ));
    }
}
