use std::{collections::HashMap, net::{TcpStream, SocketAddr, Shutdown}, io::{Read, Write}};
use chess_lib::{packet::Packet, opcode::Opcode, error::Err};
use log::{info, warn};

use crate::game::Game;

#[derive(Debug)]
pub struct Server {
    game_id: u32,
    games: HashMap<u32, Game>,
    game_users: HashMap<String, u32>,
    stream_id: HashMap<SocketAddr, String>,
    user_stream: HashMap<String, TcpStream>
}

impl Server {
    pub fn new() -> Self {
        Server {
            game_id: 0,
            games: HashMap::new(),
            game_users: HashMap::new(),
            stream_id: HashMap::new(),
            user_stream: HashMap::new()
        }
    }

    pub fn create_game(&mut self, user: &String, white: bool) -> Packet {
        info!("Creating game for {}.", user);
        self.games.insert(self.game_id, Game::new(user.to_string(), white));
        self.game_users.insert(user.to_string(), self.game_id);
        let packet = Packet::new(Opcode::CreateGameResp, self.game_id.to_string());
        self.game_id+=1;
        packet
    }

    pub fn join(&mut self, stream: &TcpStream) -> Result<String, String> {
        match stream.peer_addr() {
            Ok(addr) => {
                match self.get_packet(stream) {
                    Ok(pkt) => {
                        let user = pkt.payload_str();
                        info!("Creating user {} for {}", user, addr);
                        self.stream_id.insert(addr, user.to_string());
                        match stream.try_clone() {
                            Ok(str) => {self.user_stream.insert(user.to_string(), str);},
                            Err(e) => return Err(e.to_string())
                        }
                        
                        Ok(user)
                    },
                    Err(e) => Err(e)
                }
                
            },
            Err(e) => Err(e.to_string())
        }
        
    }

    pub fn process_packet(&mut self, user: String, packet: Packet) {
        let resp = match packet.op() {
            Opcode::CreateGame => self.create_game(&user, true),
            Opcode::KeepAlive => todo!(),
            Opcode::ListGames => self.list_games(),
            Opcode::JoinGame => self.join_game(&user, packet),
            Opcode::LeaveGame => todo!(),
            Opcode::SendMove => todo!(),
            Opcode::RecvMove => todo!(),
            Opcode::SendMsg => todo!(),
            _ => Packet::error(Err::IllegalOpcode)
        };
        self.send_packet_to_user(user, resp);
    }

    pub fn list_games(&mut self) -> Packet {
        let mut string = "No games currently.".to_string();
        if self.games.keys().count() > 1 {
            string = "Games:\n".to_string();
            for (id, game) in &self.games {
                string.push_str(&format!("{}: {}\n", id, game.status()));
            }
        }
        Packet::new(Opcode::ListGamesResp, string)
    }

    pub fn join_game(&mut self, user:& String, packet: Packet) -> Packet {
        let pl = packet.payload();
        if pl.len() != 4 {
            return Packet::error(Err::MalformedPacket)
        }
        let id: u32 = u32::from_be_bytes(pl.try_into().unwrap());
        match self.games.get_mut(&id) {
            Some(g) => {
                match self.game_users.get(user) {
                    Some(id) => Packet::error(Err::AlreadyInGame),
                    None => {
                        self.game_users.insert(user.to_string(), id);
                        Packet::new(Opcode::JoinGameResp, id.to_string())
                    }
                }
            }
            None => Packet::error(Err::GameDoesntExist)
        }
    }

    pub fn get_user(&mut self, stream: &TcpStream) -> Result<String, String> {
        match stream.peer_addr() {
            Ok(addr) => {
                info!("Stream from {}", addr);
                match self.stream_id.get(&addr) {
                    Some(user) => {
                        info!("User found: {}", user);
                        Ok(user.to_string())
                    },
                    None => {
                        let mut buf: [u8; 1] = [0];
                        match stream.peek(&mut buf) {
                            Ok(v) => {
                                if buf[0] == 2 {
                                    self.join(stream)
                                } else {
                                    return Err("User has not joined.".to_string())
                                }
                            },
                            Err(e) => Err(e.to_string())
                        }
                    }
                }
            },
            Err(e) => Err(e.to_string())
        }
    }

    pub fn get_packet(&mut self, stream: &TcpStream) -> Result<Packet, String> {
        let mut buf: [u8; 2] = [0, 0];
        match stream.peek(&mut buf) {
            Ok(_) => {
                let size: u64 = buf[1] as u64;
                let mut pbytes: Vec<u8> = Vec::new();
                match stream.take(size+2).read_to_end(&mut pbytes) {
                    Ok(_) => Packet::from_bytes(pbytes),
                    Err(e) => Err(e.to_string())
                }
            },
            Err(e) => {
                warn!("Error: {}", e.to_string());
                Err(e.to_string())
            }
        }
    }

    pub fn send_packet(&self, mut stream: &TcpStream, pkt: Packet) -> Result<(), String> {
        info!("Sending packet {:?}", pkt);
        match stream.write(&pkt.as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string())
        }
    }

    pub fn send_packet_to_user(&mut self, user: String, pkt: Packet) -> Result<(), String> {
        match self.user_stream.get(&user) {
            Some(stream) => self.send_packet(&stream, pkt),
            None => {
                Err("No user found".to_string())
            }
        }
    }

    pub fn process_stream(&mut self, stream: &TcpStream) -> Result<(), String> {
        info!("Server Status: {:?}", self);
        match self.get_user(&stream) {
            Ok(user) => {
                info!("Found user: {}", user);
                loop {
                    match self.get_packet(&stream) {
                        Ok(pkt) => self.process_packet(user.clone(), pkt),
                        Err(e) => {
                            match self.game_users.get(&user) {
                                Some(id) => {
                                        //forfeit games
                                    },
                                None => {}
                            }
                            self.stream_id.remove(&stream.peer_addr().unwrap());
                            self.user_stream.remove(&user);
                            return Err(e)
                        }
                    }
                    info!("Server Status: {:?}", self);
                }
            }
            Err(e) => Err(e)
        }
    }
}