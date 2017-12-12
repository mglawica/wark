#[derive(Debug, Default, Options)]
pub struct Options {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(help="a glob pattern for lithos files to check \
                    in current container",
              meta="PAT", no_short)]
    pub lithos_configs: String,

    #[options(help="only check. Returns error code when something \
        needs to be mutated (i.e. dir created)")]
    pub check: bool,
}
