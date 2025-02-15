use std::future::Future;
use std::marker::PhantomData;
use std::task::{Context, Poll};

use futures::future::{ok, Ready};

use crate::{IntoService, IntoServiceFactory, Service, ServiceFactory};

/// Create `ServiceFactory` for function that can act as a `Service`
pub fn service_fn<F, Fut, Req, Res, Err, Cfg>(
    f: F,
) -> FnServiceFactory<F, Fut, Req, Res, Err, Cfg>
where
    F: FnMut(Req) -> Fut + Clone,
    Fut: Future<Output = Result<Res, Err>>,
{
    FnServiceFactory::new(f)
}

pub fn service_fn2<F, Fut, Req, Res, Err>(f: F) -> FnService<F, Fut, Req, Res, Err>
where
    F: FnMut(Req) -> Fut,
    Fut: Future<Output = Result<Res, Err>>,
{
    FnService::new(f)
}

/// Create `ServiceFactory` for function that can produce services
pub fn factory_fn<F, Cfg, Srv, Fut, Err>(f: F) -> FnServiceNoConfig<F, Cfg, Srv, Fut, Err>
where
    Srv: Service,
    F: Fn() -> Fut,
    Fut: Future<Output = Result<Srv, Err>>,
{
    FnServiceNoConfig::new(f)
}

/// Create `ServiceFactory` for function that can produce services with configuration
pub fn factory_fn_cfg<F, Fut, Cfg, Srv, Err>(f: F) -> FnServiceConfig<F, Fut, Cfg, Srv, Err>
where
    F: Fn(&Cfg) -> Fut,
    Fut: Future<Output = Result<Srv, Err>>,
    Srv: Service,
{
    FnServiceConfig::new(f)
}

pub struct FnService<F, Fut, Req, Res, Err>
where
    F: FnMut(Req) -> Fut,
    Fut: Future<Output = Result<Res, Err>>,
{
    f: F,
    _t: PhantomData<Req>,
}

impl<F, Fut, Req, Res, Err> FnService<F, Fut, Req, Res, Err>
where
    F: FnMut(Req) -> Fut,
    Fut: Future<Output = Result<Res, Err>>,
{
    pub(crate) fn new(f: F) -> Self {
        Self { f, _t: PhantomData }
    }
}

impl<F, Fut, Req, Res, Err> Clone for FnService<F, Fut, Req, Res, Err>
where
    F: FnMut(Req) -> Fut + Clone,
    Fut: Future<Output = Result<Res, Err>>,
{
    fn clone(&self) -> Self {
        Self::new(self.f.clone())
    }
}

impl<F, Fut, Req, Res, Err> Service for FnService<F, Fut, Req, Res, Err>
where
    F: FnMut(Req) -> Fut,
    Fut: Future<Output = Result<Res, Err>>,
{
    type Request = Req;
    type Response = Res;
    type Error = Err;
    type Future = Fut;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Req) -> Self::Future {
        (self.f)(req)
    }
}

impl<F, Fut, Req, Res, Err> IntoService<FnService<F, Fut, Req, Res, Err>> for F
where
    F: FnMut(Req) -> Fut,
    Fut: Future<Output = Result<Res, Err>>,
{
    fn into_service(self) -> FnService<F, Fut, Req, Res, Err> {
        FnService::new(self)
    }
}

pub struct FnServiceFactory<F, Fut, Req, Res, Err, Cfg>
where
    F: FnMut(Req) -> Fut,
    Fut: Future<Output = Result<Res, Err>>,
{
    f: F,
    _t: PhantomData<(Req, Cfg)>,
}

impl<F, Fut, Req, Res, Err, Cfg> FnServiceFactory<F, Fut, Req, Res, Err, Cfg>
where
    F: FnMut(Req) -> Fut + Clone,
    Fut: Future<Output = Result<Res, Err>>,
{
    fn new(f: F) -> Self {
        FnServiceFactory { f, _t: PhantomData }
    }
}

impl<F, Fut, Req, Res, Err, Cfg> Clone for FnServiceFactory<F, Fut, Req, Res, Err, Cfg>
where
    F: FnMut(Req) -> Fut + Clone,
    Fut: Future<Output = Result<Res, Err>>,
{
    fn clone(&self) -> Self {
        Self::new(self.f.clone())
    }
}

impl<F, Fut, Req, Res, Err, Cfg> ServiceFactory for FnServiceFactory<F, Fut, Req, Res, Err, Cfg>
where
    F: FnMut(Req) -> Fut + Clone,
    Fut: Future<Output = Result<Res, Err>>,
{
    type Request = Req;
    type Response = Res;
    type Error = Err;

    type Config = Cfg;
    type Service = FnService<F, Fut, Req, Res, Err>;
    type InitError = ();
    type Future = Ready<Result<Self::Service, Self::InitError>>;

    fn new_service(&self, _: &Cfg) -> Self::Future {
        ok(FnService::new(self.f.clone()))
    }
}

impl<F, Fut, Req, Res, Err, Cfg>
    IntoServiceFactory<FnServiceFactory<F, Fut, Req, Res, Err, Cfg>> for F
where
    F: Fn(Req) -> Fut + Clone,
    Fut: Future<Output = Result<Res, Err>>,
{
    fn into_factory(self) -> FnServiceFactory<F, Fut, Req, Res, Err, Cfg> {
        FnServiceFactory::new(self)
    }
}

/// Convert `Fn(&Config) -> Future<Service>` fn to NewService
pub struct FnServiceConfig<F, Fut, Cfg, Srv, Err>
where
    F: Fn(&Cfg) -> Fut,
    Fut: Future<Output = Result<Srv, Err>>,
    Srv: Service,
{
    f: F,
    _t: PhantomData<(Fut, Cfg, Srv, Err)>,
}

impl<F, Fut, Cfg, Srv, Err> FnServiceConfig<F, Fut, Cfg, Srv, Err>
where
    F: Fn(&Cfg) -> Fut,
    Fut: Future<Output = Result<Srv, Err>>,
    Srv: Service,
{
    fn new(f: F) -> Self {
        FnServiceConfig { f, _t: PhantomData }
    }
}

impl<F, Fut, Cfg, Srv, Err> ServiceFactory for FnServiceConfig<F, Fut, Cfg, Srv, Err>
where
    F: Fn(&Cfg) -> Fut,
    Fut: Future<Output = Result<Srv, Err>>,
    Srv: Service,
{
    type Request = Srv::Request;
    type Response = Srv::Response;
    type Error = Srv::Error;

    type Config = Cfg;
    type Service = Srv;
    type InitError = Err;
    type Future = Fut;

    fn new_service(&self, cfg: &Cfg) -> Self::Future {
        (self.f)(cfg)
    }
}

/// Converter for `Fn() -> Future<Service>` fn
pub struct FnServiceNoConfig<F, C, S, R, E>
where
    F: Fn() -> R,
    S: Service,
    R: Future<Output = Result<S, E>>,
{
    f: F,
    _t: PhantomData<C>,
}

impl<F, C, S, R, E> FnServiceNoConfig<F, C, S, R, E>
where
    F: Fn() -> R,
    R: Future<Output = Result<S, E>>,
    S: Service,
{
    fn new(f: F) -> Self {
        Self { f, _t: PhantomData }
    }
}

impl<F, C, S, R, E> ServiceFactory for FnServiceNoConfig<F, C, S, R, E>
where
    F: Fn() -> R,
    R: Future<Output = Result<S, E>>,
    S: Service,
{
    type Request = S::Request;
    type Response = S::Response;
    type Error = S::Error;
    type Service = S;
    type Config = C;
    type InitError = E;
    type Future = R;

    fn new_service(&self, _: &C) -> Self::Future {
        (self.f)()
    }
}

impl<F, C, S, R, E> Clone for FnServiceNoConfig<F, C, S, R, E>
where
    F: Fn() -> R + Clone,
    R: Future<Output = Result<S, E>>,
    S: Service,
{
    fn clone(&self) -> Self {
        Self::new(self.f.clone())
    }
}

impl<F, C, S, R, E> IntoServiceFactory<FnServiceNoConfig<F, C, S, R, E>> for F
where
    F: Fn() -> R,
    R: Future<Output = Result<S, E>>,
    S: Service,
{
    fn into_factory(self) -> FnServiceNoConfig<F, C, S, R, E> {
        FnServiceNoConfig::new(self)
    }
}
