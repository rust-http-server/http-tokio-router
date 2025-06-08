use http_tokio::{BodyReader, Request};
use crate::{resolver::ctx::ResolveContext, result::HandlerResult};

pub trait Guard: Send + Sync + 'static {
    fn check<'a, 'ctx>(&self, ctx: &'a mut ResolveContext<'ctx>) -> bool;
}

pub trait Handler: Send + Sync + 'static {
    fn handle<'a>(&self, req: &'a Request, payload: &'a BodyReader) -> HandlerResult<'a>;
}

impl<F: for<'a> Fn(&'a Request, &'a BodyReader) -> HandlerResult<'a> + Send + Sync + 'static> Handler for F {
    fn handle<'a>(&self, req: &'a Request, payload: &'a BodyReader) -> HandlerResult<'a> {
        self(req, payload)
    }
}

pub trait Resolver: Send + Sync + 'static {
    fn resolve<'a, 'ctx>(&'ctx self, ctx: &'a mut ResolveContext<'ctx>) -> Option<&'ctx dyn Handler>;
}

impl<H: Handler> Resolver for H {
    fn resolve<'a, 'ctx>(&'ctx self, ctx: &'a mut ResolveContext<'ctx>) -> Option<&'ctx dyn Handler> {
        if ctx.path_segments.is_empty() {
            return Some(self)
        }
        None
    }
}