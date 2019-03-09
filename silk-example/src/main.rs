use futures::{future, Future};
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server};
use hyper::http::response;
use silk_router::parsers::rest;
use silk_router::route_match;

type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

const GET: Method = Method::GET;

fn hello(_req: &mut Request<Body>, text: &str) -> BoxFut {
    Box::new(future::ok(Response::new(Body::from(format!("Hello, {}!", text)))))
}

fn index(_req: &mut Request<Body>) -> BoxFut {
    Box::new(future::ok(Response::new(Body::from("Hello!"))))
}

fn not_found() -> BoxFut {
    Box::new(future::ok(
        response::Builder::new()
            .status(404)
            .body(Body::from("Not Found"))
            .unwrap()
        )
    )
}

fn handle(mut req: Request<Body>) -> BoxFut {
    let uri = req.uri().clone();
    route_match!(*req.method(), uri.path().clone(),
        GET ("/hi") => hello(&mut req, "world"),
        GET ("/hi/", greetee = rest) => hello(&mut req, &greetee),
        GET ("/") => index(&mut req),
        _ => not_found()
    )
}

fn main() {
    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr)
        .serve(|| service_fn(handle))
        .map_err(|e| eprintln!("server error: {}", e));

    hyper::rt::run(server);
}
