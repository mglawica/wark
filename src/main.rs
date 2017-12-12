extern crate failure;
extern crate gumdrop;
extern crate url;
#[macro_use] extern crate gumdrop_derive;

use std::process::exit;

use gumdrop::Options;

mod options;


fn main() {
    let opts = options::Options::parse_args_default_or_exit();
    println!("Opts {:?}", opts);
}
