use futures::future::BoxFuture;
use http_tokio::extensions::Extension;
use std::{collections::HashMap, ops::{Deref, DerefMut}};
use crate::{error::HttpError, extractors::FromRequest, result::HttpResult};

#[derive(Debug, Clone)]
pub struct RequestParams {
    inner: HashMap<String, String>,
}

impl RequestParams {
    pub(crate) fn new(inner: HashMap<String, String>) -> Self {
        Self { inner }
    }

    pub fn param(&self, key: &str) -> HttpResult<String> {
        self.get(key)
            .map(Clone::clone)
            .ok_or(HttpError::new(format!("invalid/missing request path parameter {key:?}"), 500))
    }
}

impl Deref for RequestParams {
    type Target = HashMap<String, String>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for RequestParams {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a> FromRequest<'a> for RequestParams {
    type Future = BoxFuture<'a, HttpResult<Self>>;
    fn from_req(req: &'a http_tokio::Request, payload: &'a http_tokio::BodyReader) -> Self::Future {
        Box::pin(async move {
            let pars = Extension::<'a, RequestParams>::from_req(req, payload).await?;
            Ok(pars.clone())
        })
    }
}
