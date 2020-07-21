extern crate nalgebra as na;

pub mod base;
pub mod physics;
pub mod planet;

use futures::{select, StreamExt};
use quinn::{Certificate, CertificateChain, PrivateKey};
use shared::commands::ClientCommand;
use slotmap::new_key_type;
use slotmap::DenseSlotMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use tokio::sync::mpsc;

//TODO: I might need some kind of server_println!() macro or smthng

//TODO: There are a lot of .unwrap()s, it's better to get rid of them

type Ordered = shared::commands::Tick;

new_key_type! {
    pub struct ClientId;
}

struct Client {
    conn: quinn::Connection,
    ordered: mpsc::Sender<Ordered>,
    entity: hecs::Entity,
}

pub struct Server {
    clients: DenseSlotMap<ClientId, Client>,
    game: crate::base::game_manager::GameManager,
    tickrate: u8, // 255 ticks/s is probably more than enough. 90% of the servers will use 60, maybe 128, but not more
}

impl Server {
    async fn run(mut self, incoming: quinn::Incoming) {
        let mut ticks =
            tokio::time::interval(std::time::Duration::from_secs(1) / self.tickrate as u32).fuse();
        let mut incoming = incoming
            .inspect(|_conn| println!("[SERVER] Client is trying to connect to the server"))
            .buffer_unordered(16);
        println!("incoming");

        let (events_tx, events_rx) = mpsc::channel(128);
        let mut events_rx = events_rx.fuse();
        loop {
            select! {
                _ = ticks.next() => {
                    self.tick().await;
                    println!("tick");
                },
                conn = incoming.select_next_some() => {
                    self.on_connect(conn, events_tx.clone()).await;
                },
                e = events_rx.select_next_some() => {
                    self.on_event(e.0, e.1);
                }
            };
        }
    }

    async fn tick(&mut self) {
        let (spawns, positions) = self.game.step();
        // Send tick info to each client
        for (_client_id, client) in &mut self.clients {
            client
                .ordered
                .send(shared::commands::Tick {
                    spawns: spawns.clone(),
                    positions: positions.clone(),
                })
                .await
                .unwrap();
        }
    }

    fn on_event(&self, client_id: ClientId, event: ClientCommand) {
        // TODO: Move to GameManager
        let player = self.clients[client_id].entity;
        let mut player = self
            .game
            .world
            .get_mut::<crate::base::player::Player>(player)
            .unwrap();
        player.state = Some(event);
    }

    async fn on_connect(
        &mut self,
        conn: Result<quinn::NewConnection, quinn::ConnectionError>,
        mut events_tx: mpsc::Sender<(ClientId, ClientCommand)>,
    ) {
        let mut conn = conn.unwrap();
        let connection = conn.connection.clone();
        let client_info = match conn.uni_streams.next().await {
            None => {
                return;
            }
            Some(stream) => {
                shared::network::receive::<shared::commands::ClientInfo>(&mut stream.unwrap())
                    .await
                    .unwrap()
            }
        };
        let (ordered_tx, mut ordered_rx) = mpsc::channel(128);

        // Take snapshot before spawning a player
        let snapshot = self.game.snapshot();
        let (eid, e) = self.game.spawn_player(client_info.clone());
        let id = self.clients.insert(Client {
            conn: connection.clone(),
            entity: e,
            ordered: ordered_tx,
        });

        let server_info = shared::commands::ServerInfo {
            character_id: eid.0,
            tickrate: self.tickrate,
            // TODO: Use actual values
            planet_seed: 1234,
            planet_radius: 720,
        };
        // Receiver thread
        tokio::spawn(async move {
            println!("[SERVER] Client has connected to the server");
            println!("[SERVER] Client info {:?}", client_info);
            let mut cmds = conn
                .uni_streams
                .map(|stream| async {
                    shared::network::receive::<shared::commands::ClientCommand>(
                        &mut stream.unwrap(),
                    )
                    .await
                    .unwrap()
                })
                .buffer_unordered(16);
            loop {
                let msg = cmds.next().await.unwrap();
                events_tx.send((id, msg)).await.unwrap();
            }
        });
        tokio::spawn(async move {
            let mut stream = connection.open_uni().await.unwrap();
            println!("[SERVER] Sending server info...");
            shared::network::send(&mut stream, &server_info).await;
            // Intial tick. Used to send snapshot
            shared::network::send(
                &mut stream,
                &shared::commands::Tick {
                    spawns: snapshot,
                    positions: vec![],
                },
            )
            .await;

            stream.finish().await.unwrap();

            let mut stream = connection.open_uni().await.unwrap();
            while let Some(command) = ordered_rx.recv().await {
                shared::network::send(&mut stream, &command).await;
            }
            stream.finish().await.unwrap();
        });
    }
}

pub fn generate_certificate() -> (CertificateChain, PrivateKey) {
    println!("[SERVER] Generating certificate...");
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    let key = cert.serialize_private_key_der();
    let cert = cert.serialize_der().unwrap();
    (
        CertificateChain::from_certs(Certificate::from_der(&cert)),
        PrivateKey::from_der(&key).unwrap(),
    )
}

pub async fn spawn() {
    let (certificate_chain, key) = generate_certificate();
    println!("Certificate Generated");
    let mut server_config = quinn::ServerConfigBuilder::default();
    server_config.certificate(certificate_chain, key).unwrap();
    println!("Server config builded");
    let mut endpoint = quinn::Endpoint::builder();
    endpoint.listen(server_config.build());
    println!("Endpoint listen");
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 2454);
    let (_, incoming) = endpoint
        .with_socket(UdpSocket::bind(&addr).unwrap())
        .unwrap();
    println!("Socket");
    let mut game = crate::base::game_manager::GameManager::new();
    println!("Game init");
    game.load_props();
    println!("Load props");
    let server = Server {
        clients: DenseSlotMap::default(),
        game: game,
        tickrate: 60,
    };
    println!("Server run");
    server.run(incoming).await;
}

#[tokio::main]
pub async fn run() {
    println!("[SERVER] Starting the server...");
    spawn().await;
}
