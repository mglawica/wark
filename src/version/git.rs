use std::ffi::OsStr;

use git2::{self, Repository, RepositoryOpenFlags, DescribeOptions, Commit};
use git2::{DescribeFormatOptions, StatusOptions, StatusShow};

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

fn count_commits(c: &Commit) -> u64 {
    let mut q = vec![c.clone()];
    let mut cnt = 0;
    while let Some(commit) = q.pop() {
        cnt += 1;
        for p in commit.parents() {
            q.push(p);
        }
    }
    cnt
}

fn get_version() -> Result<String, git2::Error> {
    let git_repo = Repository::open_ext("/work",
        RepositoryOpenFlags::empty(), &[] as &[&OsStr])?;
    let mut opt = DescribeOptions::default();
    opt.pattern("v[0-9]*");
    let mut fopt = DescribeFormatOptions::new();
    fopt.dirty_suffix("-dirty");
    match git_repo.describe(&opt) {
        Ok(ver) => return Ok(format!("{}", ver.format(Some(&fopt))?)),
        Err(ref e) if e.message().find("no reference found").is_some() => {}
        Err(e) => return Err(e),
    }
    // fallback in case no tags exists
    let commit = git_repo.head()?.peel_to_commit()?;
    let n = count_commits(&commit);
    let mut sopt = StatusOptions::new();
    sopt.show(StatusShow::IndexAndWorkdir);
    let dirty = git_repo.statuses(Some(&mut sopt))?.iter().count() > 0;
    Ok(String::from(format!("v0.0.0-{}-u{:.7}{}", n,
        commit.id().to_string(),
        if dirty { "-dirty" } else { "" })))
}
