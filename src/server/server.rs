use std::{future::Future, pin::Pin, sync::Arc};
use http_tokio::{server::{Connection, ConnectionEventsHandler, ConnectionHandler, ServerHandler}, BodyReader, Request, RequestError, Response, StatusCode};
use tokio::net::{TcpListener, ToSocketAddrs};
use crate::{server::{events::DefaultServerEvents, ServerEvents}, Router};

pub struct Server {
    keep_alive_max: usize,
    keep_alive_timeout: usize,
    events: Arc<dyn ServerEvents>
}

impl Server {
    pub fn new() -> Self {
        Server {
            events: Arc::new(DefaultServerEvents),
            keep_alive_max: 100,
            keep_alive_timeout: 5,
        }
    }

    pub fn keep_alive_max(&mut self, val: usize) -> &mut Self {
        self.keep_alive_max = val;
        self
    }

    pub fn keep_alive_timeout(&mut self, val: usize) -> &mut Self {
        self.keep_alive_timeout = val;
        self
    }

    pub fn events(&mut self, events: impl ServerEvents + 'static) {
        self.events = Arc::new(events)
    }

    pub async fn serve<A: ToSocketAddrs>(self, addr: A, router: Router) -> Result<(), std::io::Error> {
        let clone_router = ClonableRouter {
            inner: Arc::new(router),
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