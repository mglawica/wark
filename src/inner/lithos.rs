use std::io;
use std::fmt::Write;
use std::fs::{create_dir, read_link, rename};
use std::os::unix::fs::symlink;
use std::path::Path;


use inner::options::Options;
use exit::ExitCode;

use capturing_glob::{glob_with, MatchOptions};
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
    for entry in files {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                error!("Can't list dir: {}", e);
                exit.report_error();
                break;
            }
        };
        info!("Checking {:?}", entry.path());
        let res = parse_config(entry.path(),
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
        check_volume_dirs(&cfg, opt, exit);
    }
    if let Some(ref path) = opt.dns_symlinks {
        symlink_dns(&Path::new(path), opt, exit);
    }
}

fn check_volume_dirs(cfg: &ContainerConfig, opt: &Options, exit: &mut ExitCode)
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

fn symlink_dns(path: &Path, opt: &Options, exit: &mut ExitCode) {
    symlink_file("resolv.conf", path, opt, exit);
    symlink_file("hosts", path, opt, exit);
}

fn fatal_err<T>(r: Result<T, io::Error>) -> Option<io::Error> {
    match r {
        Ok(_) => None,
        Err(ref e)
        if e.kind() == io::ErrorKind::NotFound ||
           e.kind() == io::ErrorKind::InvalidInput
        => None,
        Err(e) => Some(e),
    }
}

fn symlink_file(name: &str, base: &Path, opt: &Options, exit: &mut ExitCode) {
    let dest = base.join(name);
    let spath = Path::new("/etc/").join(name);
    let spath_tmp = format!("/etc/{}.tmp", name);
    match read_link(&spath) {
        Ok(ref path) if path == &dest => {}
        Ok(ref path) if opt.check => {
            warn!("{:?} points to {:?} instead of {:?}",
                spath, path, dest);
            exit.report_error();
        }
        Err(ref e) if opt.check => {
            warn!("Error reading {:?}: {}", spath, e);
            exit.report_error();
        }
        r@Ok(_) | r@Err(_) => {
            if let Some(e) = fatal_err(r) {
                warn!("Error reading {:?}: {}", spath, e);
                exit.report_error();
            } else {
                info!("Fixing {:?} symlink to {:?}", spath, dest);
                match symlink(&dest, &spath_tmp)
                    .and_then(|()| rename(&spath_tmp, &spath))
                {
                    Ok(()) => {}
                    Err(e) => {
                        error!("Error symlinking {:?}: {}", spath, e);
                        exit.report_error();
                    }
                }
            }
        }
    }
}
