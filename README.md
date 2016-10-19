# chainmail

Strategy based authentication middleware for Iron framework.

## Install

Add this line to `Cargo.toml`:

```TOML
chainmail = { git = "https://github.com/pocket7878/chainmail.git" }
```

## Example

```rust
extern crate iron;
extern crate router;
extern crate handlebars_iron as hbs;
extern crate params;
extern crate chainmail;

use std::collections::HashMap;
use std::error::Error;
use iron::prelude::*;
use iron::status;
use hbs::{Template, HandlebarsEngine, DirectorySource};
use chainmail::strategy::{Strategy, AuthError};
use chainmail::{ChainmailMiddleware, ChainmailReqExt, AuthedUser};
use router::{Router, url_for};
use std::sync::Arc;

struct SampleAuthStrategy {
    name: String,
    pass: String
}

impl Strategy<u32> for SampleAuthStrategy {
    fn is_valid(&self, req: &mut Request) -> bool {
        use params::{Params};
        let map = req.get_ref::<Params>().unwrap();
        return match (map.find(&["name"]), map.find(&["pass"])) {
            (Some(_), Some(_)) => true,
            _ => false
        }
    }

    fn authenticate(&self, req: &mut Request) -> Result<u32, AuthError> {
        use params::{Params, Value};
        let map = req.get_ref::<Params>().unwrap();
        return match (map.find(&["name"]), map.find(&["pass"])) {
            (Some(&Value::String(ref name)), Some(&Value::String(ref pass))) if *name == self.name && *pass == self.pass => Ok(32),
            _ => Err(AuthError::new("Illigal user name or password"))
        }
    }
}

fn main() {

    fn signin_handler(req: &mut Request) -> IronResult<Response> {
        let mut resp = Response::new();
        let mut data = HashMap::new();
        data.insert(String::from("login_url"), format!("{}", url_for(req, "login", HashMap::new())));
        resp.set_mut(Template::new("signin_page", data)).set_mut(status::Ok);
        return Ok(resp);
    }

    fn login_handler(req: &mut Request) -> IronResult<Response> {
        let mut resp = Response::new();
        let mut data = HashMap::new();
        let ref current_user: Option<AuthedUser<u32>> = *req.current_user();
        match *current_user {
            Some(ref authed_user) => {
                data.insert(String::from("msg"), format!("Login success with user: {}", authed_user.user));
            },
            None => {
                data.insert(String::from("msg"), format!("Login failed"));
            }
        }
        resp.set_mut(Template::new("login_result", data)).set_mut(status::Ok);
        return Ok(resp);
    }

    //Create Router
    let mut router = Router::new();
    router.get("/sign_in", signin_handler, "signin");
    router.post("/sign_in", login_handler, "login");

    //Create Chain
    let mut chain = Chain::new(router);
    // Add HandlerbarsEngine to middleware Chain
    let mut hbse = HandlebarsEngine::new();
    hbse.add(Box::new(
        DirectorySource::new("./src/templates/", ".hbs")));
    if let Err(r) = hbse.reload() {
        panic!("{}", r.description());
    }
    chain.link_after(hbse);

    //ChainmailMiddleware
    let mut strategy_map: HashMap<String, Arc<Box<Strategy<u32> + Send + Sync>>> = HashMap::new();
    strategy_map.insert(String::from("sample"), Arc::new(Box::new(SampleAuthStrategy {
        name: String::from("sample_user"),
        pass: String::from("my_passwd")
    })));
    let chainmail_middleware = ChainmailMiddleware::new(strategy_map);
    chain.link_before(chainmail_middleware);

    println!("Listen on localhost:3000");
    Iron::new(chain).http("localhost:3000").unwrap();
}

```
