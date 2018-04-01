use std::iter::Peekable;
use std::str::Chars;

trait UrlParser {
    fn parse_from_url(Peekable<Chars>) -> Self;
}

impl UrlParser for i32 {
    fn parse_from_url(iter : Peekable<Chars>) -> i32 {
        42
    }
}

macro_rules! urlmatch {
    ($iter:expr, $( $verb:ident ( $( $url:tt )+ ) => $body:expr )+) => (
        let url_iter = $iter.chars().peekable();
        println!("Oh hi: {:?}", url_iter);
        $(
            println!("Verb: {:?}", $verb);
            urlchunks!(url_iter, $($url)+);
            println!("Body: {:?}", $body);
        )+
    )
}

macro_rules! urlchunks {
    ($iter:ident, $first:tt) => (
        println!("{:?}", $first);
    );
    ($iter:ident, $first:tt, $( $rest:tt )+) => (
        println!("{:?}", $first);
        urlchunks!($iter, $($rest)+);
    );
    ($iter:ident, $first:ident : $ty:ty) => (
        let $first : $ty = UrlParser::parse_from_url($iter);
        println!("{:?} <== {:?}", stringify!($first), stringify!($ty));
    );
    ($iter:ident, $first:ident : $ty:ty, $( $rest:tt )+) => (
        let $first : $ty = UrlParser::parse_from_url($iter);
        println!("{:?}", stringify!($first));
        urlchunks!($iter, $($rest)+);
    )
}

const GET: &'static str = "GET";
const POST: &'static str = "POST";

fn main() {
    let url = "/foo";

    urlmatch!(url,
        GET ("/foo/bar", u:i32) => { 
            println!("Neat: {}", u);
            5
        }
        POST ("/foo/bar") => { 10 }
    );
}
