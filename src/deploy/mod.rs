use std::str::from_utf8;
use std::process::{Command, Stdio, exit};
use std::collections::{BTreeSet, BTreeMap, HashMap};

pub mod config;
pub mod spec;
mod tools;

pub use self::config::{Config, Stage};
pub use self::spec::{Spec, parse_spec_or_exit};

use exit::ExitCode;


#[derive(Debug)]
struct Container {
    version: String,
}

#[derive(Debug)]
struct Context {
    spec: Spec,
    dry_run: bool,
    deployment: String,
    containers: BTreeMap<String, Container>,
}


fn check_ver(s: &str) -> bool {
    s.len() > 0 && s.chars().all(|x| {
        x.is_ascii() && x.is_alphanumeric() || x == '-' || x == '.'
    })
}

pub fn main(config: Config, deployment: String, dry_run: bool,
            vars: HashMap<String, String>)
    -> !
{
    let spec = parse_spec_or_exit(config);
    let mut code = ExitCode::new();
    let mut failed = BTreeSet::new();
    let mut context = Context {
        spec, dry_run, deployment,
        containers: BTreeMap::new(),
    };

    let deployment = match context.spec.deployments.get(&context.deployment) {
        Some(d) => d,
        None => {
            error!("No deployment {:?} found", context.deployment);
            exit(1);
        }
    };
    let containers = deployment.commands.values().map(|x| &x.container)
        .chain(deployment.daemons.values().map(|x| &x.container));
    for container in containers {
        let dep_container = format!("{}{}",
            container, context.spec.config.container_suffix);
        if context.containers.contains_key(&dep_container) {
            continue;
        }
        let output = Command::new("vagga")
            .arg("_capsule").arg("build").arg(&dep_container)
            .arg("--print-version")
            .stderr(Stdio::inherit())
            .stdout(Stdio::piped())
            .output();
        let ver_bytes = match output.as_ref().map(|x| (x.status, &x.stdout)) {
            Ok((s, ver_bytes)) if s.success() => ver_bytes,
            Ok((s, _)) => {
                error!("Container {:?} failed to build with status: {}",
                    dep_container, s);
                code.report_error();
                failed.insert(dep_container.clone());
                continue;
            }
            Err(e) => {
                error!("Can't build container {:?}: {}", dep_container, e);
                code.report_error();
                failed.insert(dep_container.clone());
                continue;
            }
        };
        let version = match from_utf8(ver_bytes) {
            Ok(s) if check_ver(&s.trim()) => s.trim().to_string(),
            _ => {
                error!("Invalid version returned for container {:?}: {:?}",
                    dep_container, String::from_utf8_lossy(&ver_bytes));
                code.report_error();
                failed.insert(dep_container.clone());
                continue;
            }
        };
        context.containers.insert(dep_container.clone(), Container {
            version: version,
        });
    }

    info!("Built containers {:?}",
        context.containers.values().map(|x| &x.version).collect::<Vec<_>>());
    if !code.is_ok() {
        error!("Failed containers {:?}", failed);
    }
    code.exit_if_failed();

    for item in &context.spec.config.script {
        match *item {
            Stage::Ciruela(ref settings) => {
                match tools::ciruela::execute(&context, settings, &vars) {
                    Ok(()) => {}
                    Err(e) => {
                        error!("Version {:?} failed to deploy: {}",
                            context.spec.version, e);
                        exit(1);
                    }
                }
            }
            Stage::VerwalterKokkupanek(ref settings) => {
                match tools::kokkupanek::execute(&context, settings, &vars) {
                    Ok(()) => {}
                    Err(e) => {
                        error!("Version {:?} failed to deploy: {}",
                            context.spec.version, e);
                        exit(1);
                    }
                }
            }
        }
    }

    if dry_run {
        info!("DRY-RUN: Version {:?} is ready for deploy",
            context.spec.version);
    } else {
        info!("Version {:?} is successfully deployed", context.spec.version);
    }
    exit(0);
}

