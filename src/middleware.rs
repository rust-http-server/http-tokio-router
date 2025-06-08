use std::sync::Arc;
use futures::future::BoxFuture;
use http_tokio::{BodyReader, Request, Response};

pub type Next<'a> = Arc<dyn Fn() -> BoxFuture<'a, Response> + Send + Sync + 'a>;

pub trait Middleware: Send + Sync + 'static + std::fmt::Debug {
    fn handle<'a>(self: Arc<Self>, req: &'a Request, payload: &'a BodyReader, next: Next<'a>) -> BoxFuture<'a, Response>;
}

pub type MiddlewareStack = Vec<Arc<dyn Middleware>>;