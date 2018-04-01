extern crate abstract_ns;
extern crate capturing_glob;
extern crate ciruela;
extern crate difference;
extern crate dir_signature;
extern crate env_logger;
extern crate futures;
extern crate lithos_shim;
extern crate ns_env_config;
extern crate quire;
extern crate rand;
extern crate semver;
extern crate serde;
extern crate serde_json;
extern crate ssh_keys;
extern crate tk_easyloop;
extern crate tk_http;
extern crate tokio_core;
extern crate trimmer;
extern crate url;
extern crate void;
#[macro_use] extern crate failure;
#[macro_use] extern crate structopt;
#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate matches;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate trimmer_derive;
#[cfg(feature="git")] extern crate git2;


use std::collections::HashMap;
use std::process::exit;
use structopt::StructOpt;

mod base;
mod deploy;
mod download;
mod exit;
mod inner;
mod local;
mod options;
mod templates;
mod version;
mod wark_version;

use std::env;

fn config(path: &Option<String>) -> deploy::Config {
    let path = match *path {
        Some(ref path) => path,
        None => {
            eprintln!("--destination is required");
            exit(1);
        }
    };
    download::download(path, true)
    .and_then(|path| deploy::Config::parse(&path))
    .unwrap_or_else(|e| {
        eprintln!("{}", e);
        ::std::process::exit(1);
    })
}

fn main() {
    use options::Command::*;

    if env::var("WARK_LOG").is_err() {
        env::set_var("WARK_LOG", "info");
    }
    env_logger::init_from_env("WARK_LOG");

    let opts = options::Options::from_args();
    let ref dest = opts.destination;
    match opts.command {
        Some(Inner(sub)) => inner::main(sub),
        Some(Check(sub)) => local::check(sub, config(dest)),
        Some(Update(sub)) => local::update(sub, config(dest)),
        None if opts.deployment.is_some() => {
            let mut vars = HashMap::new();
            for pair in &opts.var {
                let mut iter = pair.splitn(2, '=');
                match (iter.next(), iter.next()) {
                    (Some(key), Some(val)) => {
                        vars.insert(key.to_string(), val.to_string());
                    }
                    (Some(key), None) => {
                        vars.insert(key.to_string(), "true".to_string());
                    }
                    _ => unreachable!(),
                }
            }
            deploy::main(config(dest), opts.deployment.unwrap(),
                opts.dry_run, vars)
        }
        None => base::main(config(dest)),
    }
}
