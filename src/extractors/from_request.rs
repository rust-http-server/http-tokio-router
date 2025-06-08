use crate::result::HttpResult;
use futures::future::BoxFuture;
use http_tokio::{BodyReader, Request};

pub trait FromRequest<'a, Output = Self>: Sized {
    type Future: std::future::Future<Output = HttpResult<Output>>;
    fn from_req(req: &'a Request, payload: &'a BodyReader) -> Self::Future;
}

impl<'a> FromRequest<'a> for &'a Request {
    type Future = BoxFuture<'a, HttpResult<Self>>;
    fn from_req(req: &'a Request, _: &'a BodyReader) -> Self::Future {
        Box::pin(async move { Ok(&*req) })
    }
}

impl<'a> FromRequest<'a> for &'a BodyReader {
    type Future = BoxFuture<'a, HttpResult<Self>>;
    fn from_req(_: &'a Request, payload: &'a BodyReader) -> Self::Future {
        Box::pin(async move { Ok(payload) })
    }
}
