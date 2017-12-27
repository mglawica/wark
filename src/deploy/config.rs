use std::path::Path;
use std::collections::BTreeMap;

use failure::{Error, err_msg};
use quire::{parse_config, Options};
use quire::validate::{Structure, Scalar, Enum, Nothing, Mapping, Anything};
use trimmer::{Variable, Output, DataError};

use wark_version::MinimumVersion;
use templates::Pattern;


#[derive(Debug, Deserialize)]
#[serde(rename_all="kebab-case")]
pub enum VersionKind {
    GitDescribe,
}

#[derive(Debug, Deserialize)]
#[serde(tag="tool", rename_all="snake_case")]
pub enum Stage {
    CiruelaUpload { hosts: Vec<Pattern>, dir: Pattern },
    VerwalterMetadataV2 {},
}

#[derive(Debug, Deserialize, Variable)]
pub struct Config {
    pub minimum_wark: String,
    pub deployment_dirs: String,
    pub lithos_configs: String,
    pub default_copy: BTreeMap<String, String>,
    pub config_files_inner: String,
    pub vagga_config: String,
    pub container_suffix: String,
    pub version: VersionKind,
    pub deployment_name: Pattern,
    pub process_name: Pattern,
    pub script: Vec<Stage>,
}


impl Config {
    fn validator<'x>() -> Structure<'x> {
        Structure::new()
        .member("minimum_wark", MinimumVersion)
        .member("deployment_dirs", Scalar::new().default("config/deploy-(*)"))
        .member("lithos_configs", Scalar::new().default("lithos.(*).yaml"))
        .member("default_copy", Mapping::new(Scalar::new(), Scalar::new()))
        .member("config_files_inner",
            Scalar::new().default("/config/deploy-*/lithos.*.yaml"))
        .member("vagga_config", Scalar::new().default("vagga/deploy.yaml"))
        .member("container_suffix", Scalar::new().default("-deploy"))
        .member("deployment_name", Scalar::new()
            .default("{{ patterns.deployment_dirs[1] }}"))
        .member("process_name", Scalar::new()
            .default("{{ patterns.lithos_configs[1] }}"))
        .member("version", Enum::new()
            .option("git-describe", Nothing)
            .allow_plain())
        // TODO(tailhook)
        .member("script", Anything)
    }
    pub fn parse<P: AsRef<Path>>(fname: P) -> Result<Config, Error> {
        let cfg = parse_config(fname, &Config::validator(),
                               &Options::default())
            // TODO(tailhook) fix when quire fixed
            .map_err(|e| err_msg(format!("{}", e)))?;
        Ok(cfg)
    }
}

impl<'render> Variable<'render> for VersionKind {
    fn typename(&self) -> &'static str {
        "VersionKind"
    }
    fn output(&self) -> Result<Output, DataError> {
        use self::VersionKind::*;
        match *self {
            GitDescribe => Ok(Output::owned("git-describe")),
        }
    }
}

impl<'render> Variable<'render> for Stage {
    fn typename(&self) -> &'static str {
        "Stage"
    }
}
