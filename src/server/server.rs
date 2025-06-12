use std::{future::Future, pin::Pin, sync::Arc};
use http_tokio::{server::{Connection, ConnectionEventsHandler, ConnectionHandler, ServerHandler}, BodyReader, Request, RequestError, Response, StatusCode};
use tokio::net::{TcpListener, ToSocketAddrs};
use crate::{middleware::Middleware, server::{events::DefaultServerEvents, ServerEvents}, Resolver, Router};

pub struct Server {
    router: Router,
    keep_alive_max: usize,
    keep_alive_timeout: usize,
    events: Arc<dyn ServerEvents>
}

impl Server {
    pub fn new() -> Self {
        Server {
            router: Router::new(),
            events: Arc::new(DefaultServerEvents),
            keep_alive_max: 100,
            keep_alive_timeout: 5,
        }
    }

    pub fn with_config(events: impl ServerEvents + 'static) -> Self {
        Server {
            router: Router::new(),
            events: Arc::new(events),
            keep_alive_max: 100,
            keep_alive_timeout: 5,
        }
    }

    pub fn add(mut self, srv: impl Resolver) -> Self {
        self.router = self.router.add(srv);
        self
    }

    pub fn at(mut self, pattern: &str, srv: impl Resolver) -> Self {
        self.router = self.router.at(pattern, srv);
        self
    }

    pub fn wrap(mut self, middleware: impl Middleware) -> Self {
        self.router = self.router.wrap(middleware);
        self
    }

    pub async fn run<A: ToSocketAddrs>(self, addr: A) -> Result<(), std::io::Error> {
        let clone_router = ClonableRouter {
            inner: Arc::new(self.router),
            events: self.events.clone(),
        };
        let server = TcpListener::bind(addr).await?;
        loop {
            match server.accept().await {
                Ok((stream, addr)) => {
                    let conn = Connection::new(stream, addr)
                        .keep_alive_max(self.keep_alive_max)
                        .keep_alive_timeout(self.keep_alive_timeout);
                    tokio::task::spawn(conn.handle_with(clone_router.clone()));
                }
                Err(err) => self.events.on_connection_error(err)
            }
        }
    }
}

#[derive(Clone)]
struct ClonableRouter {
    inner: Arc<Router>,
    events: Arc<dyn ServerEvents>,
}

impl<'a> ServerHandler<'a> for ClonableRouter {}

impl<'a> ConnectionHandler<'a> for ClonableRouter {
    fn handle(&'a self, request: &'a Request, payload: &'a BodyReader) -> Pin<Box<dyn Future<Output = Response> + Send + 'a>> {
        Box::pin(async move { self.inner.handle_request(request, payload).await })
    }
}

impl ConnectionEventsHandler for ClonableRouter {
    fn handle_client_error(&self, err: RequestError, status_code: StatusCode) -> std::pin::Pin<Box<dyn Future<Output = Response> + Send>> {
        self.events.handle_client_error(err, status_code)
    }

    fn handle_timeout(&self) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        self.events.handle_timeout()
    }
}