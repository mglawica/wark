extern crate failure;
extern crate gumdrop;
extern crate url;
#[macro_use] extern crate gumdrop_derive;

use std::process::exit;

mod options;


fn main() {
    let opt = match options::Config::parse_args_or_exit() {
        Ok(cfg) => cfg,
        Err(e) => {
            let mut errors = e.causes();
            eprintln!("Error: {}", errors.next().unwrap());
            for e in errors {
                eprintln!("  caused by: {}", e);
            }
            exit(1);
        }
    };
    println!("Opts {:?}", opt);
}
