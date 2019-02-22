use futures::{future, Future};
use hyper::{Body, Method, Response, Request, Server};
use hyper::service::service_fn;
use silk_router::route_match;
use silk_router::parsers::{num, rest};

type BoxFut = Box<Future<Item=Response<Body>, Error=hyper::Error> + Send>;

const GET : &Method = &Method::GET;

fn hello(_req: &mut Request<Body>) -> BoxFut {
    Box::new(future::ok(Response::new(Body::from("Hi!"))))
}

fn handle(mut req: Request<Body>) -> BoxFut {
    route_match!(req.method(), req.uri().path(),
        GET ("/hi") => hello(&mut req),
        GET ("/hi", _id = num::<u32>, "/", _password = rest) => hello(&mut req),
        _ => { hello(&mut req) }
    )
}

fn main() {
    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr)
        .serve(|| service_fn(handle))
        .map_err(|e| eprintln!("server error: {}", e));

    hyper::rt::run(server);
}
