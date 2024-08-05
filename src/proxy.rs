use std::{
    io::Cursor,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use eyre::ContextCompat;
use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use rand::seq::IteratorRandom;
use rustls::{
    server::{ClientHello, ResolvesServerCert},
    sign::CertifiedKey,
    ServerConfig,
};
use tokio::{
    io::{copy_bidirectional, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

pub async fn start(addr: SocketAddr) -> eyre::Result<()> {
    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_cert_resolver(Arc::new(NullCertResolver));
    let config = Arc::new(config);
    let listener = TcpListener::bind(addr).await?;
    loop {
        match listener.accept().await {
            Ok((socket, peer_addr)) => {
                let config = config.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(config, socket, peer_addr).await {
                        eprintln!("error handling connection: {e}");
                    }
                });
            }
            Err(e) => {
                eprintln!("error accepting proxy client {e}");
            }
        }
    }
}

async fn handle_connection(
    config: Arc<ServerConfig>,
    mut socket: TcpStream,
    peer_addr: SocketAddr,
) -> eyre::Result<()> {
    println!("connection from {peer_addr}");
    let mut buffer = Vec::with_capacity(1400);
    let n = socket.read_buf(&mut buffer).await?;
    println!("read {n} bytes from client");

    let mut tls = rustls::ServerConnection::new(config)?;
    let mut reader = Cursor::new(buffer.as_slice());
    tls.read_tls(&mut reader)?;

    let _ = tls.process_new_packets();

    let server_name = tls.server_name().context("no server name")?;
    let ip = resolve_server_name(server_name)
        .await
        .context("failed to resolve server name")?;
    println!("resolved {server_name} to {ip}");

    let upstream_addr = SocketAddr::new(ip, 443);
    println!("connecting to {upstream_addr}");

    proxy_connection(upstream_addr, &buffer, socket).await?;

    Ok(())
}

async fn proxy_connection(
    upstream_addr: SocketAddr,
    buffer: &[u8],
    mut socket: TcpStream,
) -> eyre::Result<()> {
    let mut upstream = TcpStream::connect(upstream_addr).await?;
    upstream.write_all(buffer).await?;
    copy_bidirectional(&mut upstream, &mut socket).await?;

    Ok(())
}

async fn resolve_server_name(server_name: &str) -> Option<IpAddr> {
    let resolver =
        hickory_resolver::AsyncResolver::tokio(ResolverConfig::google(), ResolverOpts::default());

    match resolver.lookup_ip(server_name).await {
        Ok(answers) => {
            let rng = &mut rand::thread_rng();
            answers.iter().choose(rng)
        }
        Err(e) => {
            eprintln!("error resolving {server_name}: {e}");
            None
        }
    }
}

#[derive(Debug)]
struct NullCertResolver;

impl ResolvesServerCert for NullCertResolver {
    fn resolve(&self, _client_hello: ClientHello) -> Option<Arc<CertifiedKey>> {
        None
    }
}
