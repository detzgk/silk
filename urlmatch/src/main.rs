macro_rules! urlmatch {
    ($ty:expr, $( $verb:ident ( $( $url:tt )+ ) => $body:expr )+) => (
        let url_iter = $ty.chars().peekable();
        println!("Oh hi: {:?}", url_iter);
        $(
            println!("Verb: {:?}", $verb);
            urlchunks!($($url)+);
            println!("Body: {:?}", $body);
        )+
    )
}

macro_rules! urlchunks {
    ($first:tt, $( $rest:tt )+) => (
        println!("{:?}", $first);
        urlchunks!($($rest)+);
    );
    ($first:tt) => (
        println!("{:?}", $first);
    );
    ($first:ident : $ty:ty) => (
        let $first : $ty = Default::default();
        println!("{:?} <== {:?}", stringify!($first), stringify!($ty));
    );
    ($first:ident : $ty:ty, $( $rest:tt )+) => (
        let $first : $ty = Default::default();
        println!("{:?}", stringify!($first));
        urlchunks!($($rest)+);
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
