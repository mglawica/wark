use inner;
use local;


#[derive(Debug, Default, StructOpt)]
pub struct Options {
    #[structopt(help="a JSON file that represents deployment config",
              long="--destination", name="URL")]
    pub destination: Option<String>,

    #[structopt(help="a deployment name to deploy now",
        name="NAME", short="d", long="deployment")]
    pub deployment: Option<String>,

    #[structopt(help="prepare everything but don't deploy", long="dry-run")]
    pub dry_run: bool,

    #[structopt(help="define variable (passed as `var.NAME` to templates)",
                name="NAME=VALUE", short="D", long="var")]
    pub var: Vec<String>,

    #[structopt(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(name="inner", help="run all preparation from inside a container")]
    Inner(inner::Options),
    #[structopt(name="check", help="check that configs are up to date (for CI)")]
    Check(local::CheckOptions),
    #[structopt(name="update", help="update generated config files")]
    Update(local::UpdateOptions),
}
