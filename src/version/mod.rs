use deploy::Config;
use exit::ExitCode;


#[cfg(feature="git")]
mod git;

#[cfg(feature="git")]
use version::git::describe as git_describe;


pub fn get(cfg: &Config, exit: &mut ExitCode) -> String {
    use deploy::config::VersionKind::*;
    match cfg.version {
        GitDescribe => git_describe(exit),
    }
}

#[cfg(not(feature="git"))]
fn git_describe(exit: &mut ExitCode) -> String {
    eprintln!("Git version is not supported \
        (feature `git` is not compiled-in)");
    exit.report_error();
    "v0.0.0-unknown"
}
