use std::process::exit;

use deploy::{Config, parse_spec_or_exit};


pub fn main(config: Config) -> ! {
    let spec = parse_spec_or_exit(config);

    if spec.deployments.len() > 0 {
        println!("Available deployments:");
        for (name, dep) in &spec.deployments {
            println!("    {:10} [daemons: {}, commands: {}]",
                name, dep.daemons.len(), dep.commands.len());
        }
    }
    exit(0);
}
