use std::fmt;

use trimmer;

use ContainerConfig;


impl fmt::Debug for ContainerConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ContainerConfig")
        .finish()
    }
}

impl<'render> trimmer::Variable<'render> for ContainerConfig {
    fn typename(&self) -> &'static str {
        "deploy::Config"
    }
}
