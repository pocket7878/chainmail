extern crate iron;

pub mod strategy;

use strategy::Strategy;
use std::collections::HashMap;
use iron::prelude::*;
use iron::{typemap, AroundMiddleware, Handler};
use iron::status::Status;
use std::any::Any;
use std::sync::Arc;

pub struct AuthedUser<T> {
    pub authed_by: String,
    pub user: T,
}

pub struct ChainmailMiddleware<T>
    where T: Send + Any
{
    strategies: HashMap<String, Arc<Box<Strategy<T> + Send + Sync>>>,
    pub failure_handler: Option<Box<Handler>>,
    pub intercept_401: bool,
    pub force: bool,
}

struct ChainmailHandler<T, H: Handler>
    where T: Send + Any
{
    chainmail_middleware: ChainmailMiddleware<T>,
    base_handler: H,
}

impl<T: Send + Any, H: Handler> Handler for ChainmailHandler<T, H> {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let auth_result: Option<AuthedUser<T>> = self.chainmail_middleware.auth(req);
        if auth_result.is_none() && self.chainmail_middleware.force {
            match self.chainmail_middleware.failure_handler {
                Some(ref hn) => return hn.handle(req),
                None => panic!("No failure handler"),
            }
        } else {
            req.extensions.insert::<ChainmailMiddleware<T>>(Arc::new(auth_result));
            let res = self.base_handler.handle(req);
            match res {
                Ok(resp) => {
                    match resp.status {
                        Some(Status::Unauthorized) if self.chainmail_middleware.intercept_401 => {
                            match self.chainmail_middleware.failure_handler {
                                Some(ref fhn) => fhn.handle(req),
                                None => panic!("No failureHandler to intercept 401"),
                            }
                        }
                        _ => Ok(resp),
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }
}

impl<T> typemap::Key for ChainmailMiddleware<T>
    where T: Send + Any
{
    type Value = Arc<Option<AuthedUser<T>>>;
}

impl<T> ChainmailMiddleware<T>
    where T: Send + Any
{
    pub fn new(strategies: HashMap<String, Arc<Box<Strategy<T> + Send + Sync>>>)
               -> ChainmailMiddleware<T> {
        ChainmailMiddleware {
            strategies: strategies,
            failure_handler: None,
            intercept_401: false,
            force: false,
        }
    }

    pub fn auth(&self, req: &mut Request) -> Option<AuthedUser<T>> {
        for (st_name, strategy) in self.strategies.clone() {
            match strategy.authenticate(req) {
                Ok(u) => {
                    return Some(AuthedUser {
                        user: u,
                        authed_by: st_name,
                    })
                }
                Err(_) => {}
            }
        }
        return None;
    }
}

impl<T: Send + Any + 'static> AroundMiddleware for ChainmailMiddleware<T> {
    fn around(self, handler: Box<Handler>) -> Box<Handler> {
        Box::new(ChainmailHandler {
            chainmail_middleware: self,
            base_handler: handler,
        }) as Box<Handler>
    }
}

pub trait ChainmailReqExt<T>
    where T: Send + Any
{
    fn current_user(&self) -> Arc<Option<AuthedUser<T>>>;
    fn is_signed_in(&self) -> bool;
}

impl<'a, 'b, T: Send + Any + 'static> ChainmailReqExt<T> for Request<'a, 'b> {
    fn current_user(&self) -> Arc<Option<AuthedUser<T>>> {
        self.extensions.get::<ChainmailMiddleware<T>>().unwrap().clone()
    }

    fn is_signed_in(&self) -> bool {
        let cur_user: Arc<Option<AuthedUser<T>>> = self.current_user();
        match *cur_user {
            Some(_) => true,
            None => false,
        }
    }
}
