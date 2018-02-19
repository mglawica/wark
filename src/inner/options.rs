#[derive(Debug, Default, StructOpt)]
#[structopt(about="inner container preparation command")]
pub struct Options {
    #[structopt(help="a glob pattern for lithos files to check \
                    in current container",
              long="lithos-configs", name="PAT")]
    pub lithos_configs: String,

    #[structopt(long="dns-symlinks",
              help="Symlinks for /etc/resolv.conf and /etc/hosts \
                    should point to this directory")]
    pub dns_symlinks: Option<String>,

    #[structopt(long="check",
        help="only check. Returns error code when something \
        needs to be mutated (i.e. dir created)")]
    pub check: bool,
}
