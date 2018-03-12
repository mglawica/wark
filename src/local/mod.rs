use std::io::{self, Read, Write};
use std::fs::{File, rename, create_dir_all};
use std::path::Path;

use difference::Changeset;
use trimmer::{Context, RenderError};
use void::ResultVoidExt;

use deploy::{Config, Spec, parse_spec_or_exit};
use failure::Error;
use exit::ExitCode;
use templates;

#[derive(Debug, Default, StructOpt)]
pub struct CheckOptions {
}

#[derive(Debug, Default, StructOpt)]
pub struct UpdateOptions {
}

pub fn check_config(spec: &Spec) -> Result<bool, Error> {
    let deploy_config = render_deploy_config(&spec)
        .map_err(|e| format_err!("{}", e))?;
    let ref filename = spec.config.vagga_config;
    let mut buf = String::with_capacity(1024);
    let mut f = match File::open(&filename) {
        Ok(f) => f,
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(e) => bail!("Can't open file {:?}: {}", filename, e),
    };
    f.read_to_string(&mut buf)
        .map_err(|e| format_err!("Error reading {:?}: {}", filename, e))?;
    return Ok(deploy_config == buf);
}

pub fn check(_options: CheckOptions, config: Config) -> ! {
    let mut exit = ExitCode::new();
    let spec = parse_spec_or_exit(config);
    let deploy_config = render_deploy_config(&spec)
        .map_err(|e| exit.fatal_error(e)).void_unwrap();
    let ref filename = spec.config.vagga_config;
    let mut buf = String::with_capacity(1024);
    File::open(&filename)
        .and_then(|mut f| f.read_to_string(&mut buf))
        .map_err(|e| exit.fatal_context(filename, e)).void_unwrap();
    if deploy_config != buf {
        exit.report_error();
        eprintln!("Config {:?} is not up to date", spec.config.vagga_config);
        println!("{}", Changeset::new(&buf, &deploy_config, "\n"));
        info!("To fix config run: vagga deploy update");
    }
    if exit.is_ok() {
        info!("Everything is fine, ready for deploy");
    }
    exit.exit();
}

fn render_deploy_config(spec: &Spec) -> Result<String, RenderError> {
    let mut ctx = Context::new();
    ctx.set("spec", spec);
    templates::VAGGA_DEPLOY_CONFIG.render(&ctx)
}

pub fn update(_options: UpdateOptions, config: Config) -> ! {
    let mut exit = ExitCode::new();
    let spec = parse_spec_or_exit(config);
    let deploy_config = render_deploy_config(&spec)
        .map_err(|e| exit.fatal_error(e)).void_unwrap();
    if let Some(dir) = Path::new(&spec.config.vagga_config).parent() {
        if !dir.is_dir() {
            create_dir_all(&dir)
            .map_err(|e| exit.fatal_error(e)).void_unwrap();
        }
    }
    let tmp_name = format!("{}.tmp", spec.config.vagga_config);
    File::create(&tmp_name)
        .and_then(|mut f| f.write_all(deploy_config.as_bytes()))
        .map_err(|e| exit.fatal_error(e)).void_unwrap();
    rename(&tmp_name, spec.config.vagga_config)
        .map_err(|e| exit.fatal_error(e)).void_unwrap();
    if exit.is_ok() {
        info!("All done, ready for deploy");
    }
    exit.exit()
}
