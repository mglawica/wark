use std::str::from_utf8;
use std::ascii::AsciiExt;
use std::process::{Command, Stdio};
use std::collections::{BTreeSet, BTreeMap};

pub mod config;
pub mod spec;

pub use self::config::Config;
pub use self::spec::{Spec, parse_spec_or_exit};

use exit::ExitCode;


fn check_ver(s: &str) -> bool {
    s.len() > 0 &&
        s.chars().all(|x| x.is_ascii() && x.is_alphanumeric() || x == '.')
}

pub fn main(config: Config, deployment: String) -> ! {
    let spec = parse_spec_or_exit(config);
    let mut exit = ExitCode::new();
    let mut built = BTreeMap::new();
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
        if built.contains_key(container) {
            continue;
        }
        let output = Command::new("vagga")
            .arg("_capsule").arg("build").arg(container).arg("--print-version")
            .stderr(Stdio::inherit())
            .stdout(Stdio::piped())
            .output();
        let ver_bytes = match output.as_ref().map(|x| (x.status, &x.stdout)) {
            Ok((s, ver_bytes)) if s.success() => ver_bytes,
            Ok((s, _)) => {
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
        };
        let version = match from_utf8(ver_bytes) {
            Ok(s) if check_ver(&s.trim()) => s.trim().to_string(),
            _ => {
                error!("Invalid version returned for container {:?}: {:?}",
                    container, String::from_utf8_lossy(&ver_bytes));
                exit.report_error();
                failed.insert(container.clone());
                continue;
            }
        };
        built.insert(container.clone(), version);
    }

    info!("Built containers {:?}", built);
    if !exit.is_ok() {
        error!("Failed containers {:?}", failed);
    }
    exit.exit_if_failed();
    eprintln!("Deploying {:?}", deployment);
    unimplemented!();
}

