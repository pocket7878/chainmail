use iron::prelude::*;
use std::any::Any;
use std::collections::HashMap;
use strategy::Strategy;
use std::sync::Arc;

pub struct Chainmail<T> where T: Send + Any {
    pub strategies: HashMap<String, Arc<Strategy<T> + Send + Sync>>
}

impl<T: Send + Any> Chainmail<T> {
    pub fn auth(&self, req: &mut Request, strategies: Vec<String>) -> Option<T> {
        for st_name in strategies {
            match self.strategies.get(&st_name) {
                Some(st) => {
                    match st.authenticate(req) {
                        Ok(u) => return Some(u),
                        Err(_) => {}
                    }
                },
                None => { panic!("No such strategy: {}", st_name)}
            }
        }
        return None
    }
}
