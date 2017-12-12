use std::fmt::Write;
use std::fs::create_dir;
use std::path::Path;

use inner::options::Options;
use exit::ExitCode;

use glob::{glob_with, MatchOptions};
use lithos_shim::{ContainerConfig};
use quire::{parse_config, Options as Quire};


fn nice_cmdline(cfg: &ContainerConfig) -> String {
    let mut res = String::with_capacity(100);
    let name_start = cfg.executable.rfind('/').map(|x| x+1).unwrap_or(0);
    let name = &cfg.executable[name_start..];
    write!(res, "{}", name).unwrap();
    for item in &cfg.arguments {
        if res.len() > 60 {
            res.truncate(60);
            res.push_str("...");
            break;
        } else {
            write!(res, " {}", item).unwrap();
        }
    }
    res.chars().flat_map(|c| c.escape_default()).collect()
}


pub fn check_configs(opt: &Options, exit: &mut ExitCode) {
    let glob_result = glob_with(&opt.lithos_configs, &MatchOptions {
            case_sensitive: true,
            require_literal_separator: true,
            require_literal_leading_dot: true,
        });
    let files = match glob_result {
        Err(e) => {
            error!("Glob error {:?}: {}", opt.lithos_configs, e);
            exit.report_error();
            return;
        }
        Ok(files) => files,
    };
    for fname in files {
        let fname = match fname {
            Ok(cfg) => cfg,
            Err(e) => {
                error!("Can't list dir: {}", e);
                exit.report_error();
                break;
            }
        };
        info!("Checking {:?}", fname);
        let res = parse_config(fname,
                &ContainerConfig::validator(), &Quire::default());
        let cfg: ContainerConfig = match res {
            Ok(cfg) => cfg,
            Err(e) => {
                error!("{}", e);
                exit.report_error();
                continue;
            }
        };
        info!("Command-line: {}", nice_cmdline(&cfg));
        check_volume_dirs(opt, &cfg, exit)
    }
}

fn check_volume_dirs(opt: &Options, cfg: &ContainerConfig, exit: &mut ExitCode)
{
    for (dir, _) in &cfg.volumes {
        if !Path::new(dir).exists() {
            if !opt.check {
                info!("Creating missing directory {:?}", dir);
                match create_dir(dir) {
                    Ok(()) => {}
                    Err(e) => {
                        error!("Error creating dir {:?}: {}", dir, e);
                        exit.report_error();
                    }
                }
            } else {
                warn!("Missing directory {:?}", dir);
                exit.report_error();
            }
        }
    }
}
