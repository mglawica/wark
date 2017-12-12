use std::env::{current_dir};

use url::Url;
use gumdrop::Options as OptTrait;
use failure::{err_msg, ResultExt, Error};

#[derive(Debug, Default, Options)]
pub struct Options {

    #[options(help = "print help message")]
    pub help: bool,

    #[options(help="a JSON file that represents deployment config",
              meta="URL", no_short)]
    pub destination: Option<String>,

    #[options(help="a comma-separatated list of containers to deploy",
              meta="A,B,C", no_short)]
    pub containers: String,
}

#[derive(Debug)]
pub struct Config {
    pub destination: Url,
    pub containers: Vec<String>,
}

impl Config {

    pub fn parse_args_or_exit() -> Result<Config, Error> {

        let opts = Options::parse_args_default_or_exit();

        let containers = opts.containers.split(',')
            .filter_map(|v| {
                let x = v.trim();
                if x.len() == 0 {
                    None
                } else {
                    Some(x.to_string())
                }
            })
            .collect::<Vec<_>>();

        for container in &containers {
            let valid_chars = container.chars().all(|x| {
                x.is_alphanumeric() || x == '-' || x == '_'
            });
            if !valid_chars {
                return Err(err_msg(
                    format!("Invalid container name {:?}", container)));
            }
        }
        if containers.len() == 0 {
            return Err(err_msg( "At least one container \
                must be specified using --containers"));
        }
        let url = opts.destination
            .ok_or(err_msg("--destination must be specified"))?;
        let base = format!("file:///{}/",
            current_dir().context("error finding out current dir")?
            .to_str().ok_or(err_msg("current dir is not utf-8"))?.to_string())
            .parse::<Url>().context("current dir can't be URLified")?;
        let destination = Url::options()
            .base_url(Some(&base))
            .parse(&url)
            .context("destination")?;
        Ok(Config {
            destination,
            containers,
        })
    }
}
