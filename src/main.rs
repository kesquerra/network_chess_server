mod game;
use tokio::{spawn, net::{TcpListener, TcpStream}, sync::{mpsc::{self, Sender, Receiver}, oneshot, broadcast}, task::JoinHandle, time::Instant};
use std::{net::SocketAddr, collections::HashMap, time::Duration};
use chess_lib::{logger::Logger, packet::Packet};
use log::{info};
mod connection;
use crate::{connection::Connection, server::Server, serverpacket::ServerPacket};
mod server;
mod serverpacket;

static LOGGER: Logger = Logger {service: "Server"};


// port 8088
// creates channels for communication, and starts the processor on a single thread, and all connections on separate threads
#[tokio::main]
async fn main() {
    Logger::init(&LOGGER).unwrap();
    println!("Starting server...");

    let port = "8088";
    let listener = TcpListener::bind("127.0.0.1:".to_string() + port).await.unwrap();
    println!("Listening on port {}...", port);
    let (sptx, sprx): (Sender<ServerPacket>, Receiver<ServerPacket>) = mpsc::channel(32);
    let (ctx, crx): (Sender<(i32, oneshot::Sender<bool>)>, Receiver<(i32, oneshot::Sender<bool>)>) = mpsc::channel(32);
    let _c = run_processor(sprx, crx);
    let mut id = 0;
    loop {
        
        let (tcp, addr) = listener.accept().await.unwrap();
        let tx = sptx.clone();
        let (otx, orx) = oneshot::channel();
        ctx.send((id, otx)).await.unwrap();
        run_conn(tcp, addr, id, tx, orx).await;
        id+=1;
        
    }
}


// each connection runs the cycle of reading packets from stream, sending to processor,
// reading response packets from processor, and sending back down the tcp stream
pub async fn run_conn(tcp: TcpStream, addr: SocketAddr, id: i32, tx: Sender<ServerPacket>, krx: oneshot::Receiver<bool>) {
    spawn(async move {
        let (dtx, drx): (broadcast::Sender<Packet>, broadcast::Receiver<Packet>) = broadcast::channel(32);
        info!("Connection: {}", addr);
        let mut connection  = Connection::new(tcp, id, dtx, drx, krx);
        match connection.join().await {
            Ok(u) => {
                info!("{} joined.", u);
                loop {
                    connection.cycle(&tx, addr, &u).await;
                }
            },
            Err(_) => return
        }
    });
}


// sole processor to process incoming packets and manage keepalive statuses
pub fn run_processor(mut prx: Receiver<ServerPacket>, mut itx: Receiver<(i32, oneshot::Sender<bool>)>) -> JoinHandle<()> {
    spawn(async move {
        let mut server = Server::new();
        let mut ktable: HashMap<i32, oneshot::Sender<bool>> = HashMap::new();
        let mut ttable: HashMap<i32, Instant> = HashMap::new();
        loop {
            match itx.try_recv() {
                Ok((addr, tx)) => {
                    ktable.insert(addr, tx);
                    ttable.insert(addr, Instant::now());
                }
                Err(_) => {}
            }
            let pkt = prx.try_recv();

            if let Ok(sp) = pkt {
                match server.process_packet(sp).await {
                    Ok(()) => {},
                    Err((addr, time)) => {
                        if ttable.contains_key(&addr) {
                            ttable.insert(addr, time);
                        }
                        
                    }
                }
            }

            let mut dels: Vec<i32> = Vec::new();
            for (k, v) in &mut ttable {
                if v.elapsed() > Duration::new(20, 0) {
                    if let Some(tx) = ktable.remove(&k) {
                        tx.send(true).unwrap();
                    }
                    dels.push(*k);
                }
            }

            for k in dels {
                ttable.remove(&k);
            }
    }})
}






