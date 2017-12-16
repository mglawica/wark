extern crate capturing_glob;
extern crate env_logger;
extern crate failure;
extern crate glob;
extern crate gumdrop;
extern crate lithos_shim;
extern crate url;
extern crate quire;
extern crate semver;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate gumdrop_derive;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;


use gumdrop::Options;

mod base;
mod deploy;
mod exit;
mod inner;
mod options;
mod wark_version;

use std::env;


fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let opts = options::Options::parse_args_default_or_exit();
    match opts.command {
        Some(options::Command::Inner(sub)) => inner::main(sub),
        None => {
            if !opts.destination.starts_with("./") {
                unimplemented!("Only local urls work for now")
            }
            let cfg = deploy::Config::parse(&opts.destination)
                .unwrap_or_else(|e| {
                    eprintln!("{}", e);
                    ::std::process::exit(1);
                });
            base::main(opts, cfg)
        }
    }
}
