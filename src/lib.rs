extern crate iron;

pub mod strategy;

use strategy::Strategy;
use std::collections::HashMap;
use iron::prelude::*;
use iron::{typemap, BeforeMiddleware, AfterMiddleware, Handler};
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
    failure_handler: Option<Box<Handler>>,
    intercept_401: bool,
}

impl<T> typemap::Key for ChainmailMiddleware<T>
    where T: Send + Any
{
    type Value = Arc<Option<AuthedUser<T>>>;
}

impl<T> ChainmailMiddleware<T>
    where T: Send + Any
{
    pub fn new(strategies: HashMap<String, Arc<Box<Strategy<T> + Send + Sync>>>) -> ChainmailMiddleware<T> {
        ChainmailMiddleware {
            strategies: strategies,
            failure_handler: None,
            intercept_401: false
         }
    }

    pub fn auth(&self, req: &mut Request) -> Option<AuthedUser<T>> {
        for (st_name, strategy) in self.strategies.clone() {
            match strategy.authenticate(req) {
                Ok(u) => {
                    return Some(AuthedUser {
                        user: u,
                        authed_by: st_name
                    })
                }
                Err(_) => {}
            }
        }
        return None;
    }
}

impl<T: Send + Any + 'static> BeforeMiddleware for ChainmailMiddleware<T> {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        let auth_result: Option<AuthedUser<T>> = self.auth(req);
        req.extensions.insert::<ChainmailMiddleware<T>>(Arc::new(auth_result));
        Ok(())
    }
}

impl<T: Send + Any + 'static> AfterMiddleware for ChainmailMiddleware<T> {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        match res.status {
            Some(Status::Unauthorized) if self.intercept_401 =>
                match self.failure_handler {
                    Some(ref fhn) => fhn.handle(req),
                    None => panic!("No failureHandler to intercept 401")
                },
            _ => Ok(res)
        }
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
            None => false
        }
    }
}
