use trimmer::{Parser, Template};


lazy_static! {
    /// This holds parser so we don't need to compile it's comlplex regexes
    /// every time
    static ref PARSER: Parser = Parser::new();
    pub static ref VAGGA_DEPLOY_CONFIG: Template = PARSER.parse(
        include_str!("templates/vagga-deploy-config.trm"))
        .expect("deploy config template can't be parsed");
}
