extern crate iron;

pub mod strategy;
pub mod chainmail;

use strategy::Strategy;
use chainmail::Chainmail;
use std::collections::HashMap;
use iron::prelude::*;
use iron::{typemap, BeforeMiddleware};
use std::any::Any;
use std::error::Error;
use std::sync::Arc;

pub struct ChainmailMiddleware<T> where T: Send + Any {
    chainmail: Arc<Chainmail<T>>
}

impl<T> typemap::Key for ChainmailMiddleware<T> where T: Send + Any {
    type Value = Arc<Chainmail<T>>;
}

impl<T> ChainmailMiddleware<T> where T: Send + Any {

    pub fn from_strategies(strategies: HashMap<String, Arc<Strategy<T> + Send + Sync>>) -> ChainmailMiddleware<T> {
        ChainmailMiddleware { 
            chainmail:
                Arc::new(
                    Chainmail {
                        strategies: strategies 
                    }
                    )
        }
    }

}

impl<T: Send + Any + 'static> BeforeMiddleware for ChainmailMiddleware<T> {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<ChainmailMiddleware<T>>(self.chainmail.clone());
        Ok(())
    }
}

pub trait ChainmailReqExt<T> where T: Send + Any {
  fn chainmail(&self) -> &Chainmail<T>;
}

impl<'a, 'b, T: Send + Any + 'static>  ChainmailReqExt<T> for Request<'a, 'b> {
  fn chainmail(&self) -> &Chainmail<T> {
    self.extensions.get::<ChainmailMiddleware<T>>().unwrap()
  }
}
