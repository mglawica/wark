use deploy::{Config, parse_spec_or_exit};

#[derive(Debug, Default, Options)]
#[options(no_short)]
pub struct CheckOptions {
}

#[derive(Debug, Default, Options)]
#[options(no_short)]
pub struct UpdateOptions {
}

pub fn check(_options: CheckOptions, config: Config) -> ! {
    let spec = parse_spec_or_exit(config);
    unimplemented!();
}

pub fn update(_options: UpdateOptions, config: Config) -> ! {
    let spec = parse_spec_or_exit(config);
    unimplemented!();
}
