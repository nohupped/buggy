use anyhow::{Context, Result};
use clap::{App, Arg};
use env_logger::{Builder, Env};
use log::{error, info};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufStream},
    net::{UnixListener, UnixStream},
    runtime,
    // time::{timeout, Duration},
};

pub static SOCK_PATH: &str = "/var/run/sock.sock";

fn main() -> Result<()> {
    Builder::from_env(Env::default().default_filter_or("info")).init();
    let clap_app = App::new("openfiles")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("checking tokio openfiles")
        .arg(
            Arg::new("worker-threads")
                .long("worker-threads")
                .takes_value(true)
                .help("number of worker threads. 0 = current_thread. >0 = worker_threads")
                .default_value("1")
                .global(true),
        )
        .get_matches();
    let threads = clap_app
        .value_of("worker-threads")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let rt = match threads {
        0 => {
            info!("running in current thread");
            runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .context("cannot create runtime")?
        }
        multi => {
            info!("worker_threads: {}", multi);
            runtime::Builder::new_multi_thread()
                .worker_threads(multi)
                .enable_all()
                .thread_name("resolver-core")
                .build()
                .context("cannot create runtime")?
        }
    };

    let handle = rt.handle();
    let _enter_guard = handle.enter();
    let _ = std::fs::remove_file(SOCK_PATH);
    let listener = UnixListener::bind(SOCK_PATH).unwrap();
    rt.block_on(async move { run_listener(listener).await });
    Ok(())
}

pub async fn run_listener(listener: UnixListener) {
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                info!("Received incoming");
                tokio::task::spawn(async move {
                    match handle_client(stream).await {
                        Ok(_) => (),
                        Err(err) => error!("error handling client, error: {}", err),
                    }
                });
            }
            Err(err) => {
                error!("error accepting connection, error: {}", err);
            }
        }
    }
}

async fn handle_client(stream: UnixStream) -> Result<()> {
    let mut buf_stream = BufStream::new(stream);
    let mut line = String::new();
    buf_stream.read_line(&mut line).await?;

    info!("Received request: {}", line);
    buf_stream.write_all(b"END\r\n").await?;
    buf_stream.shutdown().await?;
    drop(buf_stream);
    Ok(())
}
