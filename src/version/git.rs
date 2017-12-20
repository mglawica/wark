use std::ffi::OsStr;

use git2::{self, Repository, RepositoryOpenFlags, DescribeOptions};

use exit::ExitCode;


pub fn describe(exit: &mut ExitCode) -> String {
    match get_version() {
        Ok(v) => v,
        Err(e) => {
            error!("git describe error: {}", e);
            exit.report_error();
            String::from("v0.0.0-unknown")
        }
    }
}

fn get_version() -> Result<String, git2::Error> {
    let git_repo = Repository::open_ext("/work",
        RepositoryOpenFlags::empty(), &[] as &[&OsStr])?;
    let describe = git_repo.describe(&DescribeOptions::default())?;
    Ok(format!("{}", describe.format(None)?))
}
