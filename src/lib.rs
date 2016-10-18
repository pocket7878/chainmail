extern crate iron;

pub mod strategy;

use strategy::Strategy;
use std::collections::HashMap;
use iron::prelude::*;
use iron::{typemap, BeforeMiddleware};
use std::any::Any;
use std::sync::Arc;

pub struct AuthedUser<T> {
    pub authed_by: String,
    pub user: T,
}

pub struct ChainmailMiddleware<T>
    where T: Send + Any
{
    strategies: HashMap<String, Arc<Box<Strategy<T> + Send + Sync>>>
}

impl<T> typemap::Key for ChainmailMiddleware<T>
    where T: Send + Any
{
    type Value = Arc<Option<AuthedUser<T>>>;
}

impl<T> ChainmailMiddleware<T>
    where T: Send + Any
{
    pub fn from_strategies(strategies: HashMap<String, Arc<Box<Strategy<T> + Send + Sync>>>)
                           -> ChainmailMiddleware<T> {
        ChainmailMiddleware { strategies: strategies }
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

pub trait ChainmailReqExt<T>
    where T: Send + Any
{
    fn authed_user(&mut self) -> Arc<Option<AuthedUser<T>>>;
}

impl<'a, 'b, T: Send + Any + 'static> ChainmailReqExt<T> for Request<'a, 'b> {
    fn authed_user(&mut self) -> Arc<Option<AuthedUser<T>>> {
        self.extensions.get::<ChainmailMiddleware<T>>().unwrap().clone()
    }
}
