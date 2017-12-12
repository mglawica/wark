
extern crate env_logger;
extern crate failure;
extern crate glob;
extern crate gumdrop;
extern crate lithos_shim;
extern crate url;
extern crate quire;
#[macro_use] extern crate gumdrop_derive;
#[macro_use] extern crate log;


use gumdrop::Options;

mod options;
mod inner;
mod exit;

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
            unimplemented!("Main command is not ready yet");
        }
    }
}
