use inner;
use local;


#[derive(Debug, Default, Options)]
#[options(no_short)]
pub struct Options {

    #[options(help = "print help message", short="h")]
    pub help: bool,

    #[options(help="a JSON file that represents deployment config",
              meta="URL")]
    pub destination: String,

    #[options(help="a deployment name to deploy now", meta="NAME", short="d")]
    pub deployment: Option<String>,

    #[options(help="prepare everything but don't deploy")]
    pub dry_run: bool,

    #[options(help="define variable (passed as `var.NAME` to templates)",
              meta="NAME=VALUE", short="D")]
    pub var: Vec<String>,

    #[options(command)]
    pub command: Option<Command>,
}

#[derive(Debug, Options)]
pub enum Command {
    #[options(help="run all preparation from inside a container")]
    Inner(inner::Options),
    #[options(help="check that configs are up to date (for CI)")]
    Check(local::CheckOptions),
    #[options(help="update generated config files")]
    Update(local::UpdateOptions),
}
