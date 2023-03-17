use std::{net::SocketAddr};

use log::info;
use tokio::{sync::{mpsc::{Sender}, oneshot, broadcast}};

use chess_lib::{packet::Packet, opcode::Opcode};
use tokio::net::TcpStream;

use crate::serverpacket::ServerPacket;

pub struct Connection {
    tcp: TcpStream,
    id: i32,
    incoming: broadcast::Sender<Packet>,
    outgoing: broadcast::Receiver<Packet>,
    kill: oneshot::Receiver<bool>
}

impl Connection {
    pub fn new(tcp: TcpStream, id: i32, incoming: broadcast::Sender<Packet>, outgoing: broadcast::Receiver<Packet>, kill: oneshot::Receiver<bool>) -> Connection {
        Connection { tcp, id, incoming, outgoing, kill}
    }

    pub async fn read_packet(&mut self) -> Option<Packet> {
        if let Ok(pkt) = Packet::from_tcp(&mut self.tcp).await {
            Some(pkt)
        } else {
            None
        }
    }

    // original join packet process to allow connection
    pub async fn join(&mut self) -> Result<String, String> {
        match self.read_packet().await {
            Some(pkt) => {
                if pkt.op() == Opcode::Join {
                    Ok(pkt.payload_str())
                } else {
                    Err("Invalid Join Packet".to_string())
                }
            }
            _ => Err("No join packet found".to_string())
        }
    }

    // sends packets from the processor back to the tcp stream
    // contains kill-switch for keepalive timeout
    pub async fn send_packets(&mut self, user: &String) {
        if let Ok(b) = self.kill.try_recv() {
            if b {
                info!("{}: timeout", user);
                return;
            }
        }
        if let Ok(pkt) = self.outgoing.try_recv() {
            pkt.send(&mut self.tcp).await.unwrap();
        }
        
    }

    // reads incomming packets and forwards them to the processor with added information
    pub async fn read_packets(&mut self, tx: &Sender<ServerPacket>, addr: SocketAddr, user: &String) {
        match self.read_packet().await {
            Some(pkt) => {
                let spkt = ServerPacket::new(pkt, addr, self.id, user.to_string(), self.incoming.clone());
                tx.send(spkt).await.unwrap();
            },
            _ => {}
        }
    }

    // connection cycle of sending/reading packets on the tcp stream
    pub async fn cycle(&mut self, tx: &Sender<ServerPacket>, addr: SocketAddr, user: &String) {
        self.send_packets(user).await;
        self.read_packets(tx, addr, user).await;
    }
}