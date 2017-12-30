use std::str::from_utf8;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use failure::{Error, Context, ResultExt, err_msg};


pub fn download(url: &str, refresh: bool) -> Result<PathBuf, Error> {
    let mut cmd = Command::new("vagga");
    cmd.arg("_capsule");
    cmd.arg("download");
    if refresh {
        cmd.arg("--refresh");
    }
    cmd.arg(url);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::inherit());
    debug!("Running {:?}", cmd);
    let res = cmd.output()
        .with_context(|e| {
            error!("Error downloading {:?}: {}", url, e);
            Context::new("failed to run `vagga _capsule download`")
        })?;
    if !res.status.success() {
        error!("Error executing `vagga _capsule download`: {}", res.status);
        return Err(err_msg("failed to run `vagga _capsule download`"))
    }
    let path = from_utf8(&res.stdout)
        .context("vagga returned invalid path")?.trim();
    Ok(PathBuf::from(path))
}
