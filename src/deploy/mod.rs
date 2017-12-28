use std::process::Command;
use std::collections::BTreeSet;

pub mod config;
pub mod spec;

pub use self::config::Config;
pub use self::spec::{Spec, parse_spec_or_exit};

use exit::ExitCode;

pub fn main(config: Config, deployment: String) -> ! {
    let spec = parse_spec_or_exit(config);
    let mut exit = ExitCode::new();
    let mut built = BTreeSet::new();
    let mut failed = BTreeSet::new();

    let deployment = match spec.deployments.get(&deployment) {
        Some(d) => d,
        None => {
            error!("No deployment {:?} found", deployment);
            ::std::process::exit(1);
        }
    };
    let containers = deployment.commands.values().map(|x| &x.container)
        .chain(deployment.daemons.values().map(|x| &x.container));
    for container in containers {
        let st = Command::new("vagga")
            .arg("_capsule").arg("build").arg(container)
            .status();
        match st {
            Ok(s) if s.success() => { }
            Ok(s) => {
                error!("Container {:?} failed to build with status: {}",
                    container, s);
                exit.report_error();
                failed.insert(container.clone());
                continue;
            }
            Err(e) => {
                error!("Can't build container {:?}: {}", container, e);
                exit.report_error();
                failed.insert(container.clone());
                continue;
            }
        }
        built.insert(container.clone());
    }

    info!("Built containers {:?}", built);
    if !exit.is_ok() {
        error!("Failed containers containers {:?}", failed);
    }
    exit.exit_if_failed();
    eprintln!("Deploying {:?}", deployment);
    unimplemented!();
}

