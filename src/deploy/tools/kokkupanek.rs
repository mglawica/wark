use std::collections::HashMap;

use failure::Error;

use deploy::Context;


#[derive(Debug, Deserialize)]
pub struct Settings {
}

pub(in deploy) fn execute(ctx: &Context,
    set: &Settings, vars: &HashMap<String, String>)
    -> Result<(), Error>
{
    println!("Kokkupanek: settings: {:?}, vars: {:?}\nContext: {:#?}",
        set, vars, ctx);
    for (name, daemon) in &ctx.spec.deployments.get("staging").unwrap().daemons {
        println!("Daemon {:?}: {:?}", name, daemon.config.metadata);
    }
    unimplemented!();
}

