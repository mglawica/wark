use std::sync::Arc;
use std::collections::{BTreeMap, BTreeSet};

use lithos_shim::{ContainerConfig, ContainerKind};
use quire::{parse_config, Options as Quire};
use capturing_glob::{glob_with, MatchOptions};
use serde_json::Value as Json;
use trimmer::Context;

use exit::ExitCode;
use deploy::config::Config;
use templates::{GlobVar};
use version;


#[derive(Debug, Variable)]
pub struct Daemon {
    pub name: String,
    pub container: String,
    pub config: Arc<ContainerConfig>,
    pub config_path: String,
}

#[derive(Debug, Variable)]
pub struct Command {
    pub name: String,
    pub container: String,
    pub config: Arc<ContainerConfig>,
    pub config_path: String,
}

#[derive(Debug, Variable)]
pub struct Deployment {
    pub daemons: BTreeMap<String, Daemon>,
    pub commands: BTreeMap<String, Command>,
}

#[derive(Debug, Variable)]
pub struct Spec {
    pub config: Config,
    pub version: String,
    pub all_containers: BTreeSet<String>,
    pub deployments: BTreeMap<String, Deployment>,
}

fn str_kind(kind: ContainerKind) -> &'static str {
    match kind {
        ContainerKind::Command => "command",
        ContainerKind::Daemon => "daemon",
        ContainerKind::CommandOrDaemon => "command-and-daemon",
    }
}


pub fn parse_spec_or_exit(config: Config) -> Spec {
    let mut exit = ExitCode::new();
    let version = version::get(&config, &mut exit);
    debug!("Version {:?}", version);

    let mut spec = Spec {
        config: config,
        version: version,
        all_containers: BTreeSet::new(),
        deployments: BTreeMap::new(),
    };

    let gopts = MatchOptions {
        case_sensitive: true,
        require_literal_separator: true,
        require_literal_leading_dot: true,
    };

    let dir_iter = glob_with(&spec.config.deployment_dirs, &gopts)
        .unwrap_or_else(|e| exit.fatal_error(e));

    for dir_entry in dir_iter {

        let dir_entry = match dir_entry {
            Ok(dir_entry) => dir_entry,
            Err(e) => {
                exit.error(e);
                continue;
            }
        };
        let full_pattern = dir_entry.path()
            .join(&spec.config.lithos_configs);
        let dir_pattern: GlobVar = (&dir_entry).into();
        let file_iter = glob_with(
            &full_pattern.to_str().expect("path is utf-8"),
                &gopts)
            .unwrap_or_else(|e| exit.fatal_error(e));

        for entry in file_iter {

            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    exit.error(e);
                    continue;
                }
            };
            debug!("Matched {:?}", entry.path());
            let file_pattern: GlobVar = (&entry).into();
            let patterns = vec![
                ("deployment_dirs", &dir_pattern),
                ("lithos_configs", &file_pattern),
            ].into_iter().collect::<BTreeMap<_, _>>();

            let mut context = Context::new();
            context.set("patterns", &patterns);
            let deployment = match spec.config.deployment_name.render(&context)
            {
                Ok(v) => v,
                Err(e) => {
                    exit.error(e);
                    "<unknown-deployment>".to_string()
                }
            };

            let process = match spec.config.process_name.render(&context) {
                Ok(v) => v,
                Err(e) => {
                    exit.error(e);
                    "<unknown-proccess>".to_string()
                }
            };

            let config_path = entry.path().to_str().expect("valid filename")
                .to_string();
            let res = parse_config(&config_path,
                    &ContainerConfig::validator(), &Quire::default());
            let config: Arc<ContainerConfig> = match res {
                Ok(cfg) => cfg,
                Err(e) => {
                    error!("{}", e);
                    exit.error(e);
                    continue;
                }
            };
            let container = match config.metadata.get("container") {
                Some(&Json::String(ref container)) => container.clone(),
                Some(_) => {
                    exit.error(format_args!(
                        "Container in {:?} must be a string", entry.path()));
                    continue;
                }
                None => {
                    exit.error(format_args!(
                        "No container specified in {:?}", entry.path()));
                    continue;
                }
            };
            debug!("Deployment {:?}, {} {:?}, container {:?}",
                deployment, str_kind(config.kind), process, container);
            //debug!("Command-line: {}", nice_cmdline(&config));
            let dep = spec.deployments.entry(deployment.clone())
                .or_insert_with(|| Deployment {
                    daemons: BTreeMap::new(),
                    commands: BTreeMap::new(),
                });
            match config.kind {
                ContainerKind::Command => {
                    // TODO(tailhook) check for conflict
                    dep.commands.insert(process.clone(), Command {
                        name: process,
                        container: container.clone(),
                        config, config_path,
                    });
                }
                ContainerKind::Daemon => {
                    // TODO(tailhook) check for conflict
                    dep.daemons.insert(process.clone(), Daemon {
                        name: process,
                        container: container.clone(),
                        config, config_path,
                    });
                }
                ContainerKind::CommandOrDaemon => {
                    // TODO(tailhook) check for conflict
                    dep.commands.insert(process.clone(), Command {
                        name: process.clone(),
                        container: container.clone(),
                        config: config.clone(),
                        config_path: config_path.clone(),
                    });
                    dep.daemons.insert(process.clone(), Daemon {
                        name: process,
                        container: container.clone(),
                        config, config_path,
                    });
                }
            }
            spec.all_containers.insert(container);
        }
    }

    exit.exit_if_failed();
    spec
}
