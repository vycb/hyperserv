#![deny(warnings)]
#![feature(custom_derive)]
#![plugin(tojson_macros)]
#![feature(plugin)]
extern crate hyper;
extern crate env_logger;

extern crate handlebars;
extern crate rustc_serialize;

use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::path::Path;
use std::collections::BTreeMap;

use handlebars::{Handlebars, RenderError, RenderContext, Helper, Context};
use rustc_serialize::json::{Json, ToJson};
// use std::io::copy;

use hyper::{Get, Post};
use hyper::server::{Server, Request, Response};
use hyper::uri::RequestUri::AbsolutePath;

#[derive(ToJson)]
struct Team {
    name: String,
    pts: u16
}

fn make_data () -> BTreeMap<String, Json> {
    let mut data = BTreeMap::new();

    data.insert("year".to_string(), "2015".to_json());

    let teams = vec![ Team { name: "Jiangsu Sainty".to_string(),
                             pts: 43u16 },
                      Team { name: "Beijing Guoan".to_string(),
                             pts: 27u16 },
                      Team { name: "Guangzhou Evergrand".to_string(),
                             pts: 22u16 },
                      Team { name: "Shandong Luneng".to_string(),
                             pts: 12u16 } ];

    data.insert("teams".to_string(), teams.to_json());
    data
}

macro_rules! try_return(
    ($e:expr) => {{
        match $e {
            Ok(v) => v,
            Err(e) => { println!("Error: {}", e); return; }
        }
    }}
);

fn echo(req: Request, mut res: Response) {
    match req.uri {
      AbsolutePath(ref path) => match (&req.method, &path[..]) {
        (&Get, "/") | (&Get, "/echo") => {

          let mut handlebars = Handlebars::new();
          let t = load_template("tpl/template.html").ok().unwrap();
          handlebars.register_template_string("table", t).ok().unwrap();
          handlebars.register_helper("format", Box::new(format_helper));
          let data = make_data();

          try_return!(res.send(handlebars.render("table", &data).ok().unwrap().as_bytes()));
          return;
        },
        (&Post, "/") | (&Post, "/echo") => {
          try_return!(res.send(b"Hello Post /echo"));
          return;
        }, // fall through, fighting mutable borrows
        _ => {
            *res.status_mut() = hyper::NotFound;
            return;
        }
      },
      _ => {
          return;
      }
    };

    // let mut res = try_return!(res.start());
    // try_return!(copy(&mut req, &mut res));
}

fn format_helper (c: &Context, h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    let param = h.params().get(0).unwrap();
    let rendered = format!("{} pts", c.navigate(rc.get_path(), param));
    try!(rc.writer.write(rendered.into_bytes().as_ref()));
    Ok(())
}

fn load_template(name: &str) -> io::Result<String> {
    let path = Path::new(name);

    let mut file = try!(File::open(path));
    let mut s = String::new();
    try!(file.read_to_string(&mut s));
    Ok(s)
}

fn main() {
    env_logger::init().unwrap();
    let server = Server::http("wram:8080").unwrap();
    let _guard = server.handle(echo);
    println!("Listening on http://wram:8080");
}
