use fuso::{
    protocol::AsyncRecvPacket, service::Transfer, AsyncRead, AsyncWrite, FusoStream, Stream,
};

async fn t() -> impl Transfer<Output = FusoStream> {
    tokio::net::TcpStream::connect("").await.unwrap()
}

#[cfg(feature = "fuso-rt-tokio")]
#[tokio::main]
async fn main() -> fuso::Result<()> {
    use std::time::Duration;

    use fuso::{factory::Handshake, penetrate::NormalUnpacker, Socket};

    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .default_format()
        .format_module_path(false)
        .init();

    fuso::builder_server_with_tokio()
        .with_handshake(Handshake)
        .with_penetrate()
        .read_timeout(None)
        .max_wait_time(Duration::from_secs(5))
        .heartbeat_timeout(Duration::from_secs(10))
        .with_unpacker_adapter_mode()
        // .append_unpacker_adapter(SocksUnpacker)
        .append_unpacker_adapter(NormalUnpacker)
        .build()
        .bind(Socket::Tcp(([0, 0, 0, 0], 8888).into()))
        .run()
        .await
        .expect("server start failed");

    Ok(())
}

#[cfg(feature = "fuso-web")]
#[tokio::main]
async fn main() {}

#[cfg(feature = "fuso-api")]
#[tokio::main]
async fn main() {}

#[cfg(feature = "fuso-rt-smol")]
fn main() -> fuso::Result<()> {}

#[cfg(feature = "fuso-rt-custom")]
fn main() {}
