use inner;


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
    Inner(inner::Options),
}
