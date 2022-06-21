use anyhow::{Context, Result};
use tokio::{
    io::{AsyncWriteExt, BufStream},
    net::UnixStream,
    runtime,
};

pub static SOCK_PATH: &str = "/var/run/sock.sock";

fn main() -> Result<()> {
    let rt = runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("resolver-core")
        .build()
        .context("cannot create runtime")?;
    rt.block_on(run_client())?;

    Ok(())
}

async fn run_client() -> Result<()> {
    loop {
        let listener = UnixStream::connect(SOCK_PATH).await?;
        let mut buf_stream = BufStream::new(listener);
        tokio::spawn(async move {
            match buf_stream.write_all(b"foobar\r\n").await {
                Ok(_) => (),
                Err(err) => {
                    println!("write_all error:: {}", err);
                }
            };

            match buf_stream.flush().await {
                Ok(_) => (),
                Err(err) => {
                    println!("flush error:: {}", err);
                }
            };
        });
    }
}
