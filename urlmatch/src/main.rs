use std::iter::Peekable;
use std::str::Chars;

pub trait UrlParser: Sized {
    fn parse_from_url(&mut Peekable<Chars>) -> Option<Self>;
}

impl UrlParser for i32 {
    fn parse_from_url(iter : &mut Peekable<Chars>) -> Option<i32> {
        Some(42)
    }
}

fn matches(iter : &mut Peekable<Chars>, text: &'static str) -> bool {
    let mut other = text.chars();
    for ch in iter {
        if let Some(oth) = other.next() {
            if ch != oth {
                return false;
            }
        }
    }
    return false;
}

macro_rules! urlmatch {
    ($iter:expr, $( $verb:ident ( $( $url:tt )+ ) => $body:expr ),+, _ => $default:expr) => (
        {
            let mut url_iter = $iter.chars().peekable();
            branch!(url_iter, $default, $( $body, $verb, ( $( $url )+ ) ),+ )
        }
    )
}

macro_rules! branch {
    ($iter:ident, $default:expr, $body:expr, $verb:ident, ( $( $url:tt )+ ), $( $bodies:expr, $verbs:ident, ( $( $urlses:tt )+ ) )+) => {
        if let Some(result) = all_predicates!($iter, $body, $($url)+) {
            result
        } else {
            branch!($iter, $default, $( $bodies, $verbs, ( $( $urlses )+ ) ),+ )
        }
    };
    ($iter:ident, $default:expr, $body:expr, $verb:ident, ( $( $url:tt )+ ) ) => {
        if let Some(result) = all_predicates!($iter, $body, $($url)+) {
            result
        } else {
            $default
        }
    };
}

macro_rules! all_predicates {
    ($iter:ident, $body:expr, $first:tt, $( $rest:tt )+) => (
        if matches(&mut $iter, $first) {
            all_predicates!($iter, $body, $($rest)+)
        } else { None }
    );
    ($iter:ident, $body:expr, $first:tt) => (
        if matches(&mut $iter, $first) {
            Some($body)
        } else { None }
    );
    ($iter:ident, $body:expr, $first:ident : $ty:ty, $( $rest:tt )+) => (
        if let Some($first) = <$ty as UrlParser>::parse_from_url(&mut $iter) {
            all_predicates!($iter, $body, $($rest)+)
        } else { None }
    );
    ($iter:ident, $body:expr, $first:ident : $ty:ty) => (
        if let Some($first) = <$ty as UrlParser>::parse_from_url(&mut $iter) {
            Some($body)
        } else { None }
    );
}

const GET: &'static str = "GET";
const POST: &'static str = "POST";

fn main() {
    let url = "/foo/bar/10";

    let res = urlmatch!(url,
        GET ("/foo/bar", u:i32) => { 
            u
        },
        POST ("/foo/bar") => { 10 },
        _ => { 5 }
    );

    println!("Result: {}", res);
}
