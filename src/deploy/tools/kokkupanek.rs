use std::collections::HashMap;
use std::time::Duration;
use std::sync::Arc;
use std::io::BufWriter;

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
use serde_json::{to_vec, to_writer, from_slice};
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
    variables: &'a HashMap<String, String>,
}

#[derive(Debug)]
struct GetLeaderCodec(Option<oneshot::Sender<String>>);
#[derive(Debug)]
struct PostNewDeployment(Option<oneshot::Sender<()>>, Arc<Vec<u8>>);


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

    println!("Kokkupanek: settings: {:?}, vars: {:?}\nContext: {:#?}",
        set, vars, ctx);
    for (name, daemon) in &ctx.spec.deployments.get("staging").unwrap().daemons {
        println!("Daemon {:?}: {:?}", name, daemon.config.metadata);
    }

    let req = Arc::new(to_vec(&GraphqlRequest {
        query: deployment_graphql,
        variables: vars,
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
                Ok(()) => Either::A(ok(Loop::Break(()))),
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
        e.request_line("GET", "/v1/action", Version::Http11);
        e.add_header("Host", "verwalter").unwrap();
        e.add_header("Content-Type", "application/json").unwrap();
        e.add_header("User-Agent",
            concat!("wark/", env!("CARGO_PKG_VERSION"))).unwrap();
        e.add_chunked().unwrap();
        e.done_headers().unwrap();
        to_writer(BufWriter::new(&mut e), &self.1)
            .expect("can serialize query");
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
        self.0.take().expect("once").send(()).ok();
        Ok(Async::Ready(data.len()))
    }
}
