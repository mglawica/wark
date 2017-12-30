use std::collections::HashMap;
use std::process::Command;
use std::time::Instant;

use failure::{Error, err_msg, Context as Fail, ResultExt};
use trimmer::{Context as Vars};

use templates::{Pattern};
use deploy::Context;


#[derive(Debug, Deserialize)]
pub struct Settings {
    hosts: Vec<Pattern>,
    dir: Pattern,
}

pub(in deploy) fn execute(ctx: &Context,
    set: &Settings, vars: &HashMap<String, String>)
    -> Result<(), Error>
{
    let mut context = Vars::new();
    context.set("vars", vars);
    for (name, container) in &ctx.containers {
        context.set("container_name", name);
        context.set("container_version", &container.version);
        let hosts = set.hosts.iter().map(|h| {
            h.render(&context)
        }).collect::<Result<Vec<String>, _>>()
            .map_err(|e| err_msg(format!("Can't render host pattern: {}", e)))?;
        let dir = set.dir.render(&context)
            .map_err(|e| err_msg(format!("Can't render dir pattern: {}", e)))?;
        let mut cmd = Command::new("ciruela");
        cmd.arg("upload");
        cmd.arg("-d");
        cmd.arg(format!("/vagga/containers/{}", container.version));
        for h in hosts {
            cmd.arg(format!("{}:{}", h, dir));
        }

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
    }
    Ok(())
}
