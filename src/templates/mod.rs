use trimmer::{Parser, Template};

mod glob_var;
mod pattern;

pub use self::glob_var::GlobVar;
pub use self::pattern::Pattern;

lazy_static! {
    /// This holds parser so we don't need to compile it's complex regexes
    /// every time
    pub static ref PARSER: Parser = Parser::new();
    pub static ref VAGGA_DEPLOY_CONFIG: Template = PARSER.parse(
        include_str!("./vagga-deploy-config.trm"))
        .expect("deploy config template can't be parsed");
}
