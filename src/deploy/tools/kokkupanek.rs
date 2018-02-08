use std::collections::HashMap;
use std::time::Duration;

use failure::{Error, err_msg};
use futures::Future;
use futures::future::{loop_fn, Loop, Either, ok, err};
use ns_env_config;
use rand::{thread_rng, Rng};
use tk_easyloop::{self, handle, timeout};
use trimmer::{Context as Vars};

use deploy::Context;
use templates::{Pattern};


#[derive(Debug, Deserialize)]
pub struct Settings {
    hosts: Vec<Pattern>,
    slug: Pattern,
}

pub(in deploy) fn execute(ctx: &Context,
    set: &Settings, vars: &HashMap<String, String>)
    -> Result<(), Error>
{
    let mut context = Vars::new();
    context.set("vars", vars);

    let hosts = set.hosts.iter().map(|h| {
        h.render(&context)
    }).collect::<Result<Vec<String>, _>>()
        .map_err(|e| err_msg(format!("Can't render host pattern: {}", e)))?;

    if hosts.is_empty() {
        return Err(err_msg("hosts must not be empty"));
    }

    println!("Kokkupanek: settings: {:?}, vars: {:?}\nContext: {:#?}",
        set, vars, ctx);
    for (name, daemon) in &ctx.spec.deployments.get("staging").unwrap().daemons {
        println!("Daemon {:?}: {:?}", name, daemon.config.metadata);
    }

    tk_easyloop::run(move || {
        let ns = ns_env_config::init(&handle())
            .expect("name system init");
        loop_fn(0, move |iter| {
            let host = thread_rng().choose(&hosts).unwrap();
            debug!("Connecting to {:?}", host);
            ns.resolve_auto(host, 8379).map_err(|e| e.into())
            .and_then(|addr| {
                println!("ADDR {:?}", addr);
                addr.pick_one()
                    .ok_or_else(|| err_msg("could not resolve name"))
            })
            .and_then(|ip| {
                debug!("Connecting to ip {}", ip);
                Err(err_msg("unimplemented"))
            })
            .then(move |res| match res {
                Ok(()) => Either::A(ok(Loop::Break(()))),
                Err(ref e) if iter > 10 => {
                    error!("Error: {}. Bailing out...", e);
                    Either::A(err(()))
                }
                Err(ref e) => {
                    error!("Error: {}. Will retry in a second...", e);
                    Either::B(timeout(Duration::new(1, 0))
                        .map(move |()| Loop::Continue(iter+1))
                        .map_err(|_| unreachable!()))
                }
            })
        })
    }).map_err(|()| err_msg("failed to execute verwalter action"))
}

