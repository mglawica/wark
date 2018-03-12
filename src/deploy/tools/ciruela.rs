use std::collections::HashMap;
use std::fs::{File, set_permissions};
use std::io::{self, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::time::Instant;
use std::path::Path;

use failure::{Error, err_msg, Context as Fail, ResultExt};
use libflate::gzip::Decoder;
use tar::Archive;
use trimmer::{Context as Vars};

use deploy::Context;
use download::download;
use templates::{Pattern};


static DEFAULT_CIRUELA: &str = "0.5.11";


#[derive(Debug, Deserialize)]
pub struct Settings {
    clusters: Vec<Pattern>,
    dir: Pattern,
    #[serde(default="default_ciruela")]
    ciruela_version: String,
}

fn default_ciruela() -> String {
    DEFAULT_CIRUELA.to_string()
}

fn unpack_ciruela(tar: &Path) -> Result<(), Error> {
    let f = BufReader::new(File::open(&tar)?);
    let d = Decoder::new(f)?;
    let mut a = Archive::new(d);
    for entry in a.entries()? {
        let mut file = entry?;
        if file.header().path()? != Path::new("usr/bin/ciruela") {
            trace!("Skipping {:?}", file.header().path());
            continue;
        }
        io::copy(&mut file, &mut File::create("/bin/ciruela")?)?;
        set_permissions("/bin/ciruela", PermissionsExt::from_mode(0o777))?;
        return Ok(());
    }
    return Err(err_msg("ciruela binary not found in archive"));
}

pub(in deploy) fn execute(ctx: &Context,
    set: &Settings, vars: &HashMap<String, String>)
    -> Result<(), Error>
{
    let ciruela = Path::new("/bin/ciruela");
    if !ciruela.exists() && !ctx.dry_run {
        let tar = download(&format!("https://github.com/tailhook/ciruela/\
            releases/download/v{0}/ciruela-static-v{0}.tar.gz",
            set.ciruela_version), false)?;
        unpack_ciruela(&tar).context("can't unpack ciruela")?;
    }


    let mut context = Vars::new();
    context.set("vars", vars);
    let clusters = set.clusters.iter().map(|h| {
        h.render(&context)
    }).collect::<Result<Vec<String>, _>>()
        .map_err(|e| err_msg(format!("Can't render host pattern: {}", e)))?;

    let mut cmd = Command::new(ciruela);
    cmd.arg("sync");
    for (name, container) in &ctx.containers {
        context.set("container_name", name);
        context.set("container_version", &container.version);
        let dir = set.dir.render(&context)
            .map_err(|e| err_msg(format!("Can't render dir pattern: {}", e)))?;
        cmd.arg("--append-weak");
        cmd.arg(format!("/vagga/base/.roots/{}/root:{}",
                        container.version, dir));
    }
    cmd.args(&clusters);
    if ctx.dry_run {
        info!("Would run: {:?}", cmd);
    } else {
        info!("Running: {:?}", cmd);
        let start = Instant::now();
        let status = cmd.status()
            .with_context(|e| {
                error!("Ciruela error: {}", e);
                Fail::new("failed to run ciruela")
            })?;
        if status.success() {
            let dur = start.elapsed();
            if dur.as_secs() > 2 {
                info!("Upload done in {}s", dur.as_secs());
            }
        } else {
            error!("Ciruela {}", status);
            return Err(err_msg("ciruela failed"));
        }
    }
    Ok(())
}
