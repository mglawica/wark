use std::sync::Arc;
use std::collections::BTreeMap;

use lithos_shim::{ContainerConfig, ContainerKind};
use quire::{parse_config, Options as Quire};
use capturing_glob::{glob_with, MatchOptions};
use serde_json::Value as Json;

use exit::ExitCode;
use deploy::config::Config;


pub struct Daemon {
    pub name: String,
    pub container: String,
    pub config: Arc<ContainerConfig>,
}

pub struct Command {
    pub name: String,
    pub container: String,
    pub config: Arc<ContainerConfig>,
}

pub struct Deployment {
    pub daemons: BTreeMap<String, Daemon>,
    pub commands: BTreeMap<String, Command>,
}

pub struct Spec {
    pub config: Config,
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
    let mut spec = Spec {
        config: config,
        deployments: BTreeMap::new(),
    };

    let iter = glob_with(&spec.config.config_files, &MatchOptions {
        case_sensitive: true,
        require_literal_separator: true,
        require_literal_leading_dot: true,
    }).unwrap_or_else(|e| exit.fatal_error(e));

    for entry in iter {

        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                exit.error(e);
                continue;
            }
        };
        let deployment = entry
            .group(spec.config.config_path_deployment).unwrap()
            .to_str().unwrap().to_string();
        let procname = entry
            .group(spec.config.config_path_process_name).unwrap()
            .to_str().unwrap().to_string();

        debug!("Matched {:?}", entry.path());

        let res = parse_config(entry.path(),
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
            deployment, str_kind(config.kind), procname, container);
        //debug!("Command-line: {}", nice_cmdline(&config));
        let dep = spec.deployments.entry(deployment.clone())
            .or_insert_with(|| Deployment {
                daemons: BTreeMap::new(),
                commands: BTreeMap::new(),
            });
        match config.kind {
            ContainerKind::Command => {
                dep.commands.insert(procname.clone(), Command {
                    name: procname,
                    container,
                    config,
                });
            }
            ContainerKind::Daemon => {
                dep.daemons.insert(procname.clone(), Daemon {
                    name: procname,
                    container,
                    config,
                });
            }
            ContainerKind::CommandOrDaemon => {
                dep.commands.insert(procname.clone(), Command {
                    name: procname.clone(),
                    container: container.clone(),
                    config: config.clone(),
                });
                dep.daemons.insert(procname.clone(), Daemon {
                    name: procname,
                    container,
                    config,
                });
            }
        }
    }

    exit.exit_if_failed();
    spec
}
