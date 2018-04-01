use std::collections::HashMap;
use std::env::{self, home_dir};
use std::fs::File;
use std::io::{self, Read};
use std::time::{SystemTime};
use std::path::Path;

use abstract_ns::Name;
use failure::{Error, err_msg, ResultExt};
use futures::future::join_all;
use trimmer::{Context as Vars};
use tk_easyloop::{self, handle};
use ns_env_config;
use ssh_keys::PrivateKey;
use ssh_keys::openssh::parse_private_key;

use ciruela::VPath;
use ciruela::blocks::ThreadedBlockReader;
use ciruela::index::InMemoryIndexes;
use ciruela::signature::sign_upload;
use ciruela::cluster::{Config, Connection};
use deploy::Context;
use dir_signature::{v1, ScannerConfig, HashType};
use templates::{Pattern};


#[derive(Debug, Deserialize)]
pub struct Settings {
    clusters: Vec<Pattern>,
    dir: Pattern,
}

fn keys_from_file(filename: &Path, allow_non_existent: bool,
    res: &mut Vec<PrivateKey>)
    -> Result<(), Error>
{
    let mut f = match File::open(filename) {
        Ok(f) => f,
        Err(ref e)
        if e.kind() == io::ErrorKind::NotFound && allow_non_existent
        => {
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };
    let mut keybuf = String::with_capacity(1024);
    f.read_to_string(&mut keybuf)?;
    let keys = parse_private_key(&keybuf)?;
    res.extend(keys);
    Ok(())
}

fn keys_from_env(name: &str, allow_non_existent: bool,
    res: &mut Vec<PrivateKey>)
    -> Result<(), Error>
{
    let value = match env::var(name) {
        Ok(x) => x,
        Err(ref e)
        if matches!(e, &env::VarError::NotPresent) && allow_non_existent
        => {
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };
    let keys = parse_private_key(&value)?;
    res.extend(keys);
    Ok(())
}

pub fn read_keys()
    -> Result<Vec<PrivateKey>, Error>
{
    let mut private_keys = Vec::new();
    keys_from_env("CIRUELA_KEY", true, &mut private_keys)
        .context(format!("Can't read env key CIRUELA_KEY"))?;
    match home_dir() {
        Some(home) => {
            let path = home.join(".ssh/id_ed25519");
            keys_from_file(&path, true, &mut private_keys)
                .context(format!("Can't read key file {:?}", path))?;
            let path = home.join(".ssh/id_ciruela");
            keys_from_file(&path, true, &mut private_keys)
                .context(format!("Can't read key file {:?}", path))?;
            let path = home.join(".ciruela/id_ed25519");
            keys_from_file(&path, true, &mut private_keys)
                .context(format!("Can't read key file {:?}", path))?;
        }
        None if private_keys.len() == 0 => {
            warn!("Cannot find home dir. \
                Use `-i` or `-k` options to specify \
                identity (private key) explicitly.");
        }
        None => {}  // fine if there is some key, say from env variable
    }
    info!("Read {} private keys", private_keys.len());
    return Ok(private_keys)
}

pub(in deploy) fn execute(ctx: &Context,
    set: &Settings, vars: &HashMap<String, String>)
    -> Result<(), Error>
{

    let mut context = Vars::new();
    context.set("vars", vars);
    let clusters = set.clusters.iter().map(|h| {
        h.render(&context)
    }).collect::<Result<Vec<String>, _>>()
        .map_err(|e| err_msg(format!("Can't render host pattern: {}", e)))?;
    let clusters = clusters.into_iter().map(|x| x.parse())
        .collect::<Result<Vec<Name>, _>>()
        .map_err(|e| err_msg(format!("Can't parse host: {}", e)))?;

    let indexes = InMemoryIndexes::new();
    let blocks = ThreadedBlockReader::new();
    let mut uploads = Vec::new();
    let timestamp = SystemTime::now();
    let keys = read_keys()?;

    for (name, container) in &ctx.containers {
        context.set("container_name", name);
        context.set("container_version", &container.version);
        let dir = set.dir.render(&context)
            .map_err(|e| err_msg(format!("Can't render dir pattern: {}", e)))?;
        let dir = VPath::from(dir);

        let spath = format!("/vagga/base/.roots/{}/root", container.version);
        let mut cfg = ScannerConfig::new();
        cfg.auto_threads();
        cfg.hash(HashType::blake2b_256());
        cfg.add_dir(&spath, "/");
        cfg.print_progress();
        let mut index_buf = Vec::new();
        v1::scan(&cfg, &mut index_buf).context(spath.clone())?;
        let image_id = indexes.register_index(&index_buf)?;
        blocks.register_dir(&spath, &index_buf)?;

        let upload = sign_upload(&dir, &image_id, timestamp, &keys);
        uploads.push(upload);
    }

    if !ctx.dry_run {
        let config = Config::new().done();
        let res = tk_easyloop::run(|| {
            let ns = ns_env_config::init(&handle()).expect("init dns");
            join_all(clusters.iter().map(move |name| {
                let ns = ns.clone();
                let indexes = indexes.clone();
                let blocks = blocks.clone();
                let conn = Connection::new(vec![name.clone()], ns,
                    indexes, blocks, &config);
                join_all(uploads.clone().into_iter().map(move |upload| {
                    let up = conn.append(upload);
                    up.future()
                }))
            }))
        })?;
        for res in res.iter().flat_map(|x| x) {
            println!("{}", res);
        }
    }
    Ok(())
}
