use std::net::SocketAddr;

use chess_lib::packet::Packet;
use tokio::sync::{broadcast::Sender};

#[derive(Debug)]
pub struct ServerPacket {
    packet: Packet,
    user: String,
    addr: SocketAddr,
    id: i32,
    outgoing: Sender<Packet>,
}

impl ServerPacket {
    pub fn new(pkt: Packet, addr: SocketAddr, id:i32, user: String, outgoing: Sender<Packet>) -> ServerPacket {
        ServerPacket { packet: pkt, user, addr, id, outgoing}
    }

    pub fn pkt(&self) -> Packet {
        self.packet.clone()
    }

    pub fn user(&self) -> String {
        self.user.to_string()
    }

    pub fn addr(self) -> SocketAddr {
        self.addr
    }

    pub fn id(self) -> i32 {
        self.id
    }

    pub fn outgoing(&self) -> Sender<Packet> {
        self.outgoing.clone()
    }

    pub fn send(&self, pkt: Packet) {
        self.outgoing.send(pkt).unwrap();
    }
}