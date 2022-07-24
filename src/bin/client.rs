use std::sync::Arc;

use fuso::{penetrate::client::PenetrateClientFactory, Socket};

#[cfg(feature = "fuso-rt-tokio")]
#[tokio::main]
async fn main() -> fuso::Result<()> {
    use fuso::{compress::Lz4Compress, Addr, TokioPenetrateConnector};

    env_logger::builder()
        .filter_module("fuso", log::LevelFilter::Info)
        .default_format()
        .format_module_path(false)
        .init();

    Lz4Compress::new(lz4_stream);

    fuso::builder_client_with_tokio()
        .build(
            Socket::tcp(
                std::env::var("ENV_SERVE")
                    .unwrap_or(String::from("127.0.0.1:6722"))
                    .parse::<Addr>()
                    .unwrap(),
            ),
            PenetrateClientFactory {
                connector_factory: Arc::new(TokioPenetrateConnector::new().await?),
                socket: {
                    (
                        Socket::tcp(([0, 0, 0, 0], 9999)),
                        Socket::tcp(([127, 0, 0, 1], 22)),
                    )
                },
            },
        )
        .run()
        .await
}

#[cfg(feature = "fuso-web")]
#[tokio::main]
async fn main() {}

#[cfg(feature = "fuso-api")]
#[tokio::main]
async fn main() {}

#[cfg(feature = "fuso-rt-smol")]
fn main() -> fuso::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .default_format()
        .format_module_path(false)
        .init();

    smol::block_on(async move {
        use fuso::SmolPenetrateConnector;

        fuso::builder_client_with_smol()
            .build(
                Socket::Tcp(8888.into()),
                PenetrateClientFactory {
                    connector_factory: Arc::new(SmolPenetrateConnector),
                    socket: {
                        (
                            Socket::Tcp(([0, 0, 0, 0], 9999).into()),
                            Socket::Tcp(([127, 0, 0, 1], 22).into()),
                        )
                    },
                },
            )
            .run()
            .await
    })
}
