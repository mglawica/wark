use url::Url;
use gumdrop::Options as OptTrait;


#[derive(Debug, Default, Options)]
pub struct Options {

    #[options(help = "print help message")]
    pub help: bool,

    #[options(help="a JSON file that represents deployment config",
              meta="URL", no_short)]
    pub destination: Option<String>,

    #[options(command)]
    pub command: Option<Command>,
}

#[derive(Debug, Options)]
pub enum Command {
    #[options(help="run all preparation from inside a container")]
    Inner(InnerOpts),
}

#[derive(Debug, Default, Options)]
pub struct InnerOpts {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(help="a glob pattern for lithos files to check \
                    in current container",
              meta="PAT", no_short)]
    pub lithos_configs: String,
}

#[derive(Debug)]
pub struct Config {
    pub destination: Url,
}
