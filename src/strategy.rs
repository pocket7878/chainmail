use iron::prelude::*;
use std::error::Error;
use std::fmt;
use std::any::Any;

#[derive(Debug)]
pub struct AuthError {
    msg: String
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Authentication failed: {}", self.msg)
    }
}

impl Error for AuthError {
    fn description(&self) -> &str {
        &self.msg
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

pub trait Strategy<U> {
    fn is_valid(&self, req: &Request) -> bool;
    fn authenticate(&self, req: &mut Request) -> Result<U, AuthError>;
}
