use std::collections::BTreeSet;

use capturing_glob::{glob_with, MatchOptions};
use quire::{parse_config, Options as Quire};
use serde_json::Value as Json;
use lithos_shim::{ContainerConfig};

use options::Options;
use deploy::Config;
use exit::ExitCode;


pub fn main(_options: Options, config: Config) -> ! {
    let mut exit = ExitCode::new();

    let iter = glob_with(&config.config_files, &MatchOptions {
        case_sensitive: true,
        require_literal_separator: true,
        require_literal_leading_dot: true,
    }).unwrap_or_else(|e| exit.fatal_error(e));

    let mut deployments = BTreeSet::new();

    for entry in iter {

        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                exit.error(e);
                continue;
            }
        };
        let deployment = entry.group(config.config_path_deployment).unwrap();
        deployments.insert(deployment.to_str().unwrap().to_string());
        debug!("Matched {:?}", entry.path());
        debug!("Deployment {:?}, process-name {:?}",
            deployment,
            entry.group(config.config_path_process_name).unwrap());

        let res = parse_config(entry.path(),
                &ContainerConfig::validator(), &Quire::default());
        let cfg: ContainerConfig = match res {
            Ok(cfg) => cfg,
            Err(e) => {
                error!("{}", e);
                exit.error(e);
                continue;
            }
        };
        let container = match cfg.metadata.get("container") {
            Some(&Json::String(ref container)) => container,
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
        debug!("Container: {:?}", container);
        //debug!("Command-line: {}", nice_cmdline(&cfg));
    }
    if deployments.len() > 0 {
        println!("Available deployments:");
        for dep in deployments {
            println!("    {}", dep);
        }
    }

    exit.exit();
}
