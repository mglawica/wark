use capturing_glob::{glob_with, MatchOptions};

use options::Options;
use deploy::Config;
use exit::ExitCode;


pub fn main(_options: Options, config: Config) -> ! {
    let mut exit = ExitCode::new();

    let iter = glob_with(&config.config_files, &MatchOptions {
        case_sensitive: true,
        require_literal_separator: true,
        require_literal_leading_dot: true,
    }).unwrap_or_else(|e| exit.fatal_error(e));

    for entry in iter {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                exit.error(e);
                continue;
            }
        };
        debug!("Matched {:?}", entry.path());
        debug!("Deployment {:?}, process-name {:?}",
            entry.group(config.config_path_deployment).unwrap(),
            entry.group(config.config_path_process_name).unwrap());
    }

    exit.exit();
}
