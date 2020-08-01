use futures_util::StreamExt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum ServerCommand {
    Tick(shared::commands::Tick),
    ServerInfoUpdate(shared::commands::ServerInfo),
}

pub struct Client {
    pub network_sender: mpsc::UnboundedSender<shared::commands::ClientCommand>,
    pub network_receiver: mpsc::UnboundedReceiver<ServerCommand>,
}

async fn handle_out(
    connection: quinn::Connection,
    mut out_rx: mpsc::UnboundedReceiver<shared::commands::ClientCommand>,
) {
    while let Some(command) = out_rx.recv().await {
        let mut stream = connection.open_uni().await.unwrap();
        shared::network::send(&mut stream, &command).await;
        stream.finish().await.unwrap();
    }
}

#[tokio::main(core_threads = 1)]
async fn connect(
    in_tx: mpsc::UnboundedSender<ServerCommand>,
    out_rx: mpsc::UnboundedReceiver<shared::commands::ClientCommand>,
) {
    let mut endpoint = quinn::Endpoint::builder();
    let mut client_cfg = quinn::ClientConfig::default();
    let tls_cfg = std::sync::Arc::get_mut(&mut client_cfg.crypto).unwrap();
    tls_cfg
        .dangerous()
        .set_certificate_verifier(std::sync::Arc::new(AcceptAnyCertificate));
    endpoint.default_client_config(client_cfg);
    let (endpoint, _) = endpoint.bind(&"[::]:0".parse().unwrap()).unwrap();

    let mut connection = endpoint
        .connect(
            &"127.0.0.1:1234".parse::<SocketAddr>().unwrap(),
            //&SocketAddr::new(IpAddr::V4(Ipv4Addr::new(185, 161, 210, 210)), 2454),
            "localhost",
        )
        .unwrap()
        .await
        .unwrap();
    let mut stream = connection.connection.open_uni().await.unwrap();
    println!("[CLIENT] Sending client info...");
    shared::network::send(
        &mut stream,
        &shared::commands::ClientInfo {
            name: String::from(format!("player_{}", rand::random::<u16>())),
        },
    )
    .await;
    stream.finish().await.unwrap();

    println!("[CLIENT] Waiting for server info...");
    let mut stream = connection.uni_streams.next().await.unwrap().unwrap();

    let server_info = shared::network::receive::<shared::commands::ServerInfo>(&mut stream)
        .await
        .unwrap();

    in_tx
        .send(ServerCommand::ServerInfoUpdate(server_info))
        .unwrap();

    // TODO: separate snapshot and tick
    let snapshot = shared::network::receive::<shared::commands::Tick>(&mut stream)
        .await
        .unwrap();
    in_tx.send(ServerCommand::Tick(snapshot)).unwrap();

    tokio::spawn(handle_out(connection.connection, out_rx));

    let mut ordered = connection.uni_streams.next().await.unwrap().unwrap();
    loop {
        let tick = shared::network::receive::<shared::commands::Tick>(&mut ordered)
            .await
            .unwrap();
        in_tx.send(ServerCommand::Tick(tick)).unwrap();
    }
}

pub fn spawn() -> Client {
    let (in_tx, in_rx) = mpsc::unbounded_channel();
    let (out_tx, out_rx) = mpsc::unbounded_channel();
    std::thread::spawn(move || {
        connect(in_tx.clone(), out_rx);
    });

    Client {
        network_sender: out_tx,
        network_receiver: in_rx,
    }
}

struct AcceptAnyCertificate;

impl rustls::ServerCertVerifier for AcceptAnyCertificate {
    fn verify_server_cert(
        &self,
        _roots: &rustls::RootCertStore,
        _presented_certs: &[rustls::Certificate],
        _dns_name: webpki::DNSNameRef,
        _ocsp_response: &[u8],
    ) -> Result<rustls::ServerCertVerified, rustls::TLSError> {
        Ok(rustls::ServerCertVerified::assertion())
    }
}
