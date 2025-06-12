use std::{future::Future, pin::Pin, sync::Arc};
use async_fn_traits::AsyncFn2;
use http_tokio::{BodyReader, Request, Response};
use crate::{error::HttpError, extractors::RequestParams, middleware::{Middleware, Next}, resolver::{ctx::ResolveContext, node::Node, traits::{Handler, Resolver}}, result::RouteResult};

pub type NotFoundHandler = Box<
    dyn for<'a> Fn(&'a Request, &'a BodyReader) -> Pin<Box<dyn Future<Output = RouteResult> + Send + Sync + 'a>>
        + Send
        + Sync,
>;

pub type ErrorHandler = Box<
    dyn for<'a> Fn(&'a Request, HttpError) -> Pin<Box<dyn Future<Output = Response> + Send + Sync + 'a>>
        + Send
        + Sync,
>;

pub struct Router {
    root: Node,
    error_handler: Option<ErrorHandler>,
    not_found_handler: Option<NotFoundHandler>,
}

impl Router {
    pub fn new() -> Self {
        Router { 
            root: Node::new(),
            error_handler: None,
            not_found_handler: None
        }
    }

    pub fn add(mut self, srv: impl Resolver) -> Self {
        self.root = self.root.add(srv);
        self
    }

    // pub fn guard(mut self, guard: impl Guard) -> Self {
    //     self.root = self.root.guard(guard);
    //     self
    // }

    pub fn at(mut self, pattern: &str, srv: impl Resolver) -> Self {
        self.root = self.root.at(pattern, srv);
        self
    }

    pub fn wrap(mut self, middleware: impl Middleware) -> Self {
        self.root = self.root.wrap(middleware);
        self
    }

    pub fn set_error_handler<F>(mut self, handler: F) -> Self
    where 
        F: for<'a> AsyncFn2<&'a Request, HttpError, Output = Response> + Send + Sync + 'static,
        for<'a> <F as AsyncFn2<&'a Request, HttpError>>::OutputFuture: Send + Sync
    {
        self.error_handler = Some(Box::new(move |req, err| Box::pin(handler(req, err))));
        self
    }

    pub fn set_not_found_handler<F>(mut self, handler: F) -> Self
    where 
        F: for<'a> AsyncFn2<&'a Request, &'a BodyReader, Output = RouteResult> + Send + Sync + 'static, 
        for<'a> <F as AsyncFn2<&'a Request, &'a BodyReader>>::OutputFuture: Send + Sync 
    {
        self.not_found_handler = Some(Box::new(move |err, req| Box::pin(handler(err, req))));
        self
    }

    pub async fn handle_request(&self, req: &Request, payload: &BodyReader) -> Response {
        let mut resolve_ctx = ResolveContext::new(&req);
        match self.root.resolve(&mut resolve_ctx) {
            Some(handler) => {
                req.extensions.insert(RequestParams::new(resolve_ctx.params)).await;
                self.run_stack(&req, &payload, &resolve_ctx.layers, handler).await
            },
            None => match self.handle_not_found(req, payload).await {
                Ok(res) => res,
                Err(err) => self.handle_error(req, err).await,
            }
        }
    }
}

impl Router {
    async fn run_stack(&self, req: &Request, payload: &BodyReader, middlewares: &[Arc<dyn Middleware + 'static>], handler: &dyn Handler) -> Response {
        let mut next: Next<'_> = Arc::new(|| {
            Box::pin(async { 
                match handler.handle(&req, &payload).await {
                    Ok(res) => res,
                    Err(err) => self.handle_error(req, err).await,
                }
            })
        });
    
        for mw in middlewares.iter().rev() {
            let prev = next;
            let req = &req;
            let payload = &payload;
            next = Arc::new(move || {
                mw.clone().handle(req, payload, prev.clone())
            });
        }
    
        next().await
    }

    async fn handle_error(&self, req: &Request, err: HttpError) -> Response {
        match &self.error_handler {
            Some(handle_fn) => handle_fn(req, err).await,
            None => Response::build().status(err.status).body(err.message)
        }
    }

    async fn handle_not_found(&self, req: &Request, payload: &BodyReader) -> RouteResult {
        match &self.not_found_handler {
            Some(handle_fn) => handle_fn(req, payload).await,
            None => Ok(Response::build().status(404).body("404 Not Found")),
        }
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}