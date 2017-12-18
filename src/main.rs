extern crate capturing_glob;
extern crate difference;
extern crate env_logger;
extern crate failure;
extern crate gumdrop;
extern crate lithos_shim;
extern crate url;
extern crate quire;
extern crate semver;
extern crate serde;
extern crate serde_json;
extern crate trimmer;
extern crate void;
#[macro_use] extern crate gumdrop_derive;
#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate trimmer_derive;


use gumdrop::Options;

mod base;
mod deploy;
mod exit;
mod inner;
mod local;
mod options;
mod templates;
mod wark_version;

use std::env;

fn config(path: &str) -> deploy::Config {
    if !path.starts_with("./") {
        unimplemented!("Only local urls work for now")
    }
    deploy::Config::parse(&path)
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            ::std::process::exit(1);
        })
}

fn main() {
    use options::Command::*;

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let opts = options::Options::parse_args_default_or_exit();
    let ref dest = opts.destination;
    match opts.command {
        Some(Inner(sub)) => inner::main(sub),
        Some(Check(sub)) => local::check(sub, config(dest)),
        Some(Update(sub)) => local::update(sub, config(dest)),
        None => base::main(config(&dest)),
    }
}
