use std::{any::type_name, ops::{Deref, DerefMut}};
use anymap::any::Any;
use futures::future::BoxFuture;
use http_tokio::extensions::Extension;
use crate::{extractors::FromRequest, result::HttpResult};

impl<'a, T: anymap::any::IntoBox<(dyn Any + Send + Sync + 'static)>> FromRequest<'a> for Extension<'a, T> {
    type Future = BoxFuture<'a, HttpResult<Self>>;
    fn from_req(req: &'a http_tokio::Request, _: &'a http_tokio::BodyReader) -> Self::Future {
        Box::pin(async move {
            req.extensions.get::<T>().await
                .ok_or(format!("Trying to obtain unregistered extension {}", type_name::<Self>()).into())
        })
    }
}

/// An extractor for a Clone-able extension that does not implement InitExtension (so that can be potentially uninitialized)
/// 
/// For extensions that do not implement Clone, use http_tokio::Extension<'_, T>
struct Ext<T: Clone>(pub T);

impl<T: Clone> Deref for Ext<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Clone> DerefMut for Ext<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T: Clone + anymap::any::IntoBox<(dyn Any + Send + Sync + 'static)>> FromRequest<'a> for Ext<T> {
    type Future = BoxFuture<'a, HttpResult<Self>>;
    fn from_req(req: &'a http_tokio::Request, payload: &'a http_tokio::BodyReader) -> Self::Future {
        Box::pin(async move {
            let ext = Extension::<'a, T>::from_req(req, payload).await?;
            Ok(Ext(ext.clone()))
        })
    }
}

pub trait InitExtension: Clone + Send + Sync + 'static {
    fn init<'a>(req: &'a http_tokio::Request, payload: &'a http_tokio::BodyReader) -> BoxFuture<'a, HttpResult<Self>>;
}

impl <'a, T: InitExtension> FromRequest<'a> for T {
    type Future = BoxFuture<'a, HttpResult<Self>>;
    fn from_req(req: &'a http_tokio::Request, payload: &'a http_tokio::BodyReader) -> Self::Future {
        Box::pin(async move {
            // to avoid deadlock with nested init extensions, i have to acquire the lock twice
            let registered = { // use a block to drop the lock immediatly
                req.extensions.lock().await.get_mut::<T>().cloned()
            };
            
            let val = registered.unwrap_or({
                // this init call may access some other InitExtension
                let ext = T::init(req, payload).await?;
                // so i lock again after that
                req.extensions.lock().await.insert(ext.clone());
                ext
            });

            Ok(val)
        })
    }
}