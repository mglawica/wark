use std::collections::HashMap;
use std::time::Duration;
use std::sync::Arc;

use failure::{Error, err_msg, Fail};
use futures::{Future, Stream, Async, Sink};
use futures::future::{loop_fn, Loop, Either, ok, err, FutureResult, empty};
use futures::sync::oneshot;
use futures::stream::{once};
use ns_env_config;
use rand::{thread_rng, Rng};
use tk_easyloop::{self, handle, timeout};
use trimmer::{Context as Vars};
use tokio_core::net::TcpStream;
use serde_json::{to_vec, to_value, from_slice, Value as Json};
use tk_http::{Version, Status};
use tk_http::client::{RecvMode, Head, Error as HError, Encoder, EncoderDone};
use tk_http::client::{Codec, Config, Proto};

use deploy::Context;
use templates::{Pattern};


#[derive(Debug, Deserialize)]
#[serde(rename_all="kebab-case")]
pub struct Settings {
    hosts: Vec<Pattern>,
    slug: Pattern,
    deployment_graphql: Pattern,
}

#[derive(Debug, Serialize)]
struct GraphqlRequest<'a> {
    query: String,
    variables: HashMap<&'a str, Json>,
}

#[derive(Debug)]
struct GetLeaderCodec(Option<oneshot::Sender<String>>);
#[derive(Debug)]
struct PostNewDeployment(Option<oneshot::Sender<Json>>, Arc<Vec<u8>>);

#[derive(Debug, Serialize)]
pub struct NewDeployment<'a> {
    version: &'a str,
    daemons: Vec<NewDaemon<'a>>,
    commands: Vec<NewCommand<'a>>,
}

#[derive(Debug, Serialize)]
pub struct NewDaemon<'a> {
    config: &'a String,
    image: &'a String,
    //variables: Option<Vec<NewVariable>>,
}

#[derive(Debug, Serialize)]
pub struct NewCommand<'a> {
    config: &'a String,
    image: &'a String,
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
    let slug = set.slug.render(&context)
        .map_err(|e| err_msg(format!("Can't render slug pattern: {}", e)))?;
    let deployment_graphql = set.deployment_graphql.render(&context)
        .map_err(|e| err_msg(
            format!("Can't render deployemnt-graphql: {}", e)))?;

    if hosts.is_empty() {
        return Err(err_msg("hosts must not be empty"));
    }

    let dep = match ctx.spec.deployments.get(&ctx.deployment) {
        Some(dep) => dep,
        None => {
            return Err(err_msg(format!("no deployment {:?} found",
                ctx.deployment)));
        }
    };
    println!("Kokkupanek: settings: {:?}, vars: {:?}\nContext: {:#?}",
        set, vars, ctx);

    let mut gvars = HashMap::new();
    gvars.insert("slug", Json::String(slug));
    gvars.insert("config", to_value(&NewDeployment {
        version: &ctx.spec.version,
        daemons: dep.daemons.values().map(|d| Ok(NewDaemon {
            image: &ctx.containers.get(&d.container)
                .ok_or_else(|| {
                    err_msg(format!("container {:?} not found", d.container))
                })?.version,
            config: &d.config_path,
        })).collect::<Result<_, Error>>()?,
        commands: dep.commands.values().map(|c| Ok(NewCommand {
            image: &ctx.containers.get(&c.container)
                .ok_or_else(|| {
                    err_msg(format!("container {:?} not found", c.container))
                })?.version,
            config: &c.config_path,
        })).collect::<Result<_, Error>>()?,
    }).expect("new deployment serializes fine"));

    let req = Arc::new(to_vec(&GraphqlRequest {
        query: deployment_graphql,
        variables: gvars,
    }).expect("can serialize graphql request"));

    tk_easyloop::run(move || {
        let ns = ns_env_config::init(&handle())
            .expect("name system init");
        loop_fn(0, move |niter| {
            let host = thread_rng().choose(&hosts).unwrap();
            let ns = ns.clone();
            let req = req.clone();
            debug!("Connecting to {:?}", host);
            ns.resolve_auto(host, 8379).map_err(|e| e.into())
            .and_then(|addr| {
                addr.pick_one()
                    .ok_or_else(|| err_msg("could not resolve name"))
            })
            .and_then(|addr| {
                debug!("Connecting to ip {}", addr);
                TcpStream::connect(&addr, &handle())
                .map_err(move |e| {
                    err_msg(format!("error connecting to {}: {}", addr, e))
                })
            })
            .and_then(|sock| {
                let (tx, rx) = oneshot::channel();
                let proto = Proto::new(sock,
                    &handle(), &Arc::new(Config::new()));
                proto.send_all(once::<_, HError>(Ok(
                    GetLeaderCodec(Some(tx))
                )).chain(empty().into_stream()))
                .select2(rx)
                .then(|res| match res {
                    Ok(Either::B((val, _))) => Ok(val),
                    Err(Either::A((e, _))) => Err(e.into()),
                    Err(Either::B((e, _))) => Err(e.into()),
                    _ => {
                        Err(err_msg("request error")) // TODO(tailhook)
                    }
                })
            })
            .and_then(move |leader_name| {
                info!("Leader name {:?}", leader_name);
                ns.resolve_auto(&leader_name, 8379)
                .map_err(move |e| {
                    err_msg(format!(
                        "Error resolving {:?}: {}", leader_name, e))
                })
            })
            .and_then(|addr| {
                addr.pick_one()
                    .ok_or_else(|| err_msg("could not resolve leader name"))
            })
            .and_then(|addr| {
                debug!("Connecting to leader at ip {}", addr);
                TcpStream::connect(&addr, &handle())
                .map_err(move |e| {
                    err_msg(format!("error connecting to leader at {}: {}",
                        addr, e))
                })
            })
            .and_then(move |sock| {
                let (tx, rx) = oneshot::channel();
                let proto = Proto::new(sock,
                    &handle(), &Arc::new(Config::new()));
                proto.send_all(once::<_, HError>(Ok(
                    PostNewDeployment(Some(tx), req.clone())
                )).chain(empty().into_stream()))
                .select2(rx)
                .then(|res| match res {
                    Ok(Either::B((val, _))) => Ok(val),
                    Err(Either::A((e, _))) => Err(e.into()),
                    Err(Either::B((e, _))) => Err(e.into()),
                    _ => {
                        Err(err_msg("request error")) // TODO(tailhook)
                    }
                })
            })
            .then(move |res| match res {
                Ok(info) => {
                    // TODO(tailhook) figure out is it okay
                    info!("Response {:#?}", info);
                    Either::A(ok(Loop::Break(())))
                }
                Err(ref e) if niter > 20 => {
                    error!("Error: {}. Bailing out...", e);
                    Either::A(err(()))
                }
                Err(ref e) => {
                    error!("Error: {}. Will retry in a second...", e);
                    Either::B(timeout(Duration::new(1, 0))
                        .map(move |()| Loop::Continue(niter+1))
                        .map_err(|_| unreachable!()))
                }
            })
        })
    }).map_err(|()| err_msg("failed to execute verwalter action"))
}

#[derive(Debug, Fail)]
#[fail(display = "http response with status {:?}", _0)]
struct InvalidStatus(Option<Status>);

impl<S> Codec<S> for GetLeaderCodec {
    type Future = FutureResult<EncoderDone<S>, HError>;
    fn start_write(&mut self, mut e: Encoder<S>) -> Self::Future {
        e.request_line("GET", "/v1/status", Version::Http11);
        e.add_header("Host", "verwalter").unwrap();
        e.add_header("User-Agent",
            concat!("wark/", env!("CARGO_PKG_VERSION"))).unwrap();
        e.done_headers().unwrap();
        ok(e.done())
    }
    fn headers_received(&mut self, headers: &Head) -> Result<RecvMode, HError> {
        if headers.status() == Some(Status::Ok) {
            Ok(RecvMode::buffered(65536))
        } else {
            Err(HError::custom(InvalidStatus(headers.status()).compat()))
        }
    }
    fn data_received(&mut self, data: &[u8], end: bool)
        -> Result<Async<usize>, HError>
    {
        assert!(end);

        #[derive(Deserialize)]
        struct StatusInfo {
            leader: LeaderInfo,
        }
        #[derive(Deserialize)]
        struct LeaderInfo {
            name: String,
        }

        from_slice(data)
        .map_err(|e| error!("Can't deserialize data: {}", e))
        .map(|val: StatusInfo| {
            self.0.take().expect("once").send(val.leader.name).ok()
        })
        .ok();
        Ok(Async::Ready(data.len()))
    }
}

impl<S> Codec<S> for PostNewDeployment {
    type Future = FutureResult<EncoderDone<S>, HError>;
    fn start_write(&mut self, mut e: Encoder<S>) -> Self::Future {
        e.request_line("GET", "/v1/wait_action", Version::Http11);
        e.add_header("Host", "verwalter").unwrap();
        e.add_header("Content-Type", "application/json").unwrap();
        e.add_header("User-Agent",
            concat!("wark/", env!("CARGO_PKG_VERSION"))).unwrap();
        e.add_chunked().unwrap();
        e.done_headers().unwrap();
        e.write_body(&self.1);
        ok(e.done())
    }
    fn headers_received(&mut self, headers: &Head) -> Result<RecvMode, HError> {
        if headers.status() == Some(Status::Ok) {
            Ok(RecvMode::buffered(1000))
        } else {
            Err(HError::custom(InvalidStatus(headers.status()).compat()))
        }
    }
    fn data_received(&mut self, data: &[u8], end: bool)
        -> Result<Async<usize>, HError>
    {
        assert!(end);
        if data.len() == 0 {
            self.0.take().expect("once").send(Json::String("okay".into())).ok();
        } else {
            from_slice(data)
            .map_err(|e| error!("Can't deserialize data: {}", e))
            .map(|val: Json| {
                self.0.take().expect("once").send(val).ok()
            }).ok();
        }
        Ok(Async::Ready(data.len()))
    }
}
