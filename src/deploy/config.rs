use std::path::Path;

use failure::{Error, err_msg};
use quire::{parse_config, Options};
use quire::validate::{Structure, Scalar};

use wark_version::MinimumVersion;


#[derive(Debug, Deserialize, Variable)]
pub struct Config {
    pub minimum_wark: String,
    pub config_files: String,
    pub config_path_deployment: usize,
    pub config_path_process_name: usize,
    pub config_files_inner: String,
    pub vagga_config: String,
    pub container_suffix: String,
}


impl Config {
    fn validator<'x>() -> Structure<'x> {
        Structure::new()
        .member("minimum_wark", MinimumVersion)
        .member("config_files",
            Scalar::new().default("config/deploy-(*)/lithos.(*).yaml"))
        .member("config_path_deployment", Scalar::new().default(1))
        .member("config_path_process_name", Scalar::new().default(2))
        .member("config_files_inner",
            Scalar::new().default("/config/deploy-(*)/lithos.(*).yaml"))
        .member("vagga_config", Scalar::new().default("vagga/deploy.yaml"))
        .member("container_suffix", Scalar::new().default("-deploy"))
    }
    pub fn parse<P: AsRef<Path>>(fname: P) -> Result<Config, Error> {
        let cfg = parse_config(fname, &Config::validator(),
                               &Options::default())
            // TODO(tailhook) fix when quire fixed
            .map_err(|e| err_msg(format!("{}", e)))?;
        Ok(cfg)
    }
}
