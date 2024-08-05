use std::net::{Ipv4Addr, SocketAddr};

use tokio::try_join;

mod dns;
mod proxy;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let dns_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 8053);
    let dns_server = dns::start(dns_addr);

    let proxy_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 8443);
    let proxy_server = proxy::start(proxy_addr);

    try_join!(dns_server, proxy_server)?;

    Ok(())
}
