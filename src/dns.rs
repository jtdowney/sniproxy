use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

use hickory_server::{
    authority::MessageResponseBuilder,
    proto::{
        op::Header,
        rr::{
            rdata::{A, AAAA},
            RData, Record, RecordType,
        },
    },
    server::{Request, ResponseHandler, ResponseInfo},
};
use tokio::net::UdpSocket;

struct DnsHandler;

#[async_trait::async_trait]
impl hickory_server::server::RequestHandler for DnsHandler {
    async fn handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        mut response_handle: R,
    ) -> ResponseInfo {
        let header = Header::response_from_request(request.header());
        let name = request.query().original().name().clone();
        let answer = match request.query().query_type() {
            RecordType::A => {
                vec![Record::from_rdata(
                    name.clone(),
                    60,
                    RData::A(A(Ipv4Addr::LOCALHOST)),
                )]
            }
            RecordType::AAAA => {
                vec![Record::from_rdata(
                    name,
                    60,
                    RData::AAAA(AAAA(Ipv6Addr::LOCALHOST)),
                )]
            }
            _ => vec![],
        };

        let resp = MessageResponseBuilder::from_message_request(request).build(
            header,
            &answer,
            &[],
            &[],
            &[],
        );
        response_handle.send_response(resp).await.unwrap()
    }
}

pub async fn start(addr: SocketAddr) -> eyre::Result<()> {
    let udp_socket = UdpSocket::bind(addr).await?;
    let mut dns_server = hickory_server::ServerFuture::new(DnsHandler);
    dns_server.register_socket(udp_socket);
    dns_server.block_until_done().await?;
    Ok(())
}
