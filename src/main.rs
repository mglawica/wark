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
#[cfg(feature="git")] extern crate git2;


use gumdrop::Options;
use std::process::{Command, Stdio};
use std::str::from_utf8;
use std::collections::HashMap;

mod base;
mod deploy;
mod exit;
mod inner;
mod local;
mod options;
mod templates;
mod version;
mod wark_version;

use std::env;

fn config(path: &str) -> deploy::Config {
    let res = Command::new("vagga")
        .arg("_capsule")
        .arg("download")
        .arg(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .expect("error executing vagga _capsule download");
    if !res.status.success() {
        eprintln!("Error executing vagga _capsule download: {}", res.status);
        ::std::process::exit(1);
    }
    let path = from_utf8(&res.stdout).expect("valid cache path").trim();

    deploy::Config::parse(&path)
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

    let opts = options::Options::parse_args_default_or_exit();
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
