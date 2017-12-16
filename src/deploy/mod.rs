use std::path::Path;

use failure::{Error, err_msg};
use quire::{parse_config, Options};
use quire::validate::Structure;

use wark_version::MinimumVersion;


#[derive(Debug, Deserialize)]
pub struct Config {
    pub minimum_wark: String,
}


impl Config {
    fn validator<'x>() -> Structure<'x> {
        Structure::new()
        .member("minimum_wark", MinimumVersion)
    }
    pub fn parse<P: AsRef<Path>>(fname: P) -> Result<Config, Error> {
        let cfg = parse_config(fname, &Config::validator(),
                               &Options::default())
            // TODO(tailhook) fix when quire fixed
            .map_err(|e| err_msg(format!("{}", e)))?;
        Ok(cfg)
    }
}
