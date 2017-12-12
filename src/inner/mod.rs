mod options;
mod lithos;

pub use self::options::Options;

use exit::ExitCode;

pub fn main(options: Options) -> ! {
    let mut exit = ExitCode::new();
    lithos::check_configs(&options, &mut exit);
    exit.exit();
}
