use std::{collections::HashMap};
use chess_lib::{packet::Packet, opcode::Opcode, error::Err};
use log::{info};
use tokio::{sync::broadcast, time::Instant};

use crate::{game::Game, serverpacket::ServerPacket};

#[derive(Debug)]
pub struct Server {
    game_id: u32,
    games: HashMap<u32, Game>,
    game_users: HashMap<String, u32>
}

impl Server {
    pub fn new() -> Self {
        Server {
            game_id: 0,
            games: HashMap::new(),
            game_users: HashMap::new(),
        }
    }


    // create chess game, add to database and users, return response packet to send to user
    pub fn create_game(&mut self, user: &String, pc: broadcast::Sender<Packet>, white: bool) -> Packet {
        if self.game_users.contains_key(user) {
            return Packet::error(Err::AlreadyInGame)
        }
        info!("{}: create_game", user);
        let game = Game::new(user.to_string(), pc, white);
        let fen = game.fen();
        self.games.insert(self.game_id, game);
        self.game_users.insert(user.to_string(), self.game_id);
        let packet = Packet::new(Opcode::CreateGameResp, fen);
        self.game_id+=1;
        packet
    }

    // read incoming packet, determine the action, and then send the response packet
    // back to the original packets tcp stream
    pub async fn process_packet(&mut self, packet: ServerPacket) -> Result<(), (i32, Instant)> {
        let pkt = packet.pkt();
        let user = packet.user();
        let resp = match pkt.op() {
            Opcode::CreateGame => Some(self.create_game(&user, packet.outgoing(), true)),
            Opcode::ListGames => Some(self.list_games(&user)),
            Opcode::JoinGame => Some(self.join_game(&user, pkt)),
            Opcode::LeaveGame => Some(self.leave_game(&user)),
            Opcode::SendMove => Some(self.make_move(&user, pkt)),
            Opcode::ShowGame => Some(self.show_game(&user)),
            Opcode::Join => None,
            Opcode::KeepAlive => {
                info!("{}: keepalive", user);
                return Err((packet.id(), Instant::now()))
            }
            _ => Some(Packet::error(Err::IllegalOpcode))
        };
        if let Some(p) = resp {
            packet.send(p);
        }
        
        Ok(())
    }

    // get all games from database and return the response packet
    pub fn list_games(&mut self, user: &String) -> Packet {
        info!("{}: list_games", user);
        let mut string = "No games currently.".to_string();
        if self.games.keys().count() > 0 {
            string = "Games:\n".to_string();
            for (id, game) in &self.games {
                string.push_str(&format!("{}: {}\n", id, game.status()));
            }
        }
        Packet::new(Opcode::ListGamesResp, string)
    }


    // get users ongoing game and return the string of the game in a packet
    pub fn show_game(&mut self, user: &String) -> Packet {
        info!("{}: show_game", user);
        if self.game_users.contains_key(user) {
            let id = self.game_users.get(user).unwrap();
            match self.games.get(id) {
                Some(g) => Packet::new(Opcode::ShowGameResp, g.fen()),
                None => Packet::error(Err::GameDoesntExist)
            }
        } else {
            Packet::error(Err::GameDoesntExist)
        }
    }


    // add user to an existing game and return the string of the game in a packet
    pub fn join_game(&mut self, user:& String, packet: Packet) -> Packet {
        let id = match self.get_game_id(packet) {
            Ok(i) => i,
            Err(_) => return Packet::error(Err::MalformedPacket)
        };
        info!("{}: join_game {}", user, id);
        if self.games.contains_key(&id) {
            if self.game_users.contains_key(user) {
                Packet::error(Err::AlreadyInGame)
            } else {
                
                let g: &mut Game = self.games.get_mut(&id).unwrap();
                if let Ok(()) = &mut g.join(user.to_string()) {
                    self.game_users.insert(user.to_string(), id);
                    Packet::new(Opcode::JoinGameResp, g.fen())
                } else {
                    Packet::error(Err::GameLimit)
                }
                
            }
        } else {
            Packet::error(Err::GameDoesntExist)
        }
    }

    // add a move to the users current game and return the string of the new board state
    pub fn make_move(&mut self, user: &String, packet: Packet) -> Packet {
        match self.game_users.get(user) {
            Some(id) => {
                match self.games.get_mut(id) {
                    Some(g) => {
                        let m = packet.payload_str();
                        info!("{}: make_move {}", user, m);
                        match g.make_move(m) {
                            Ok(fen) => {
                                g.send(g.is_white_turn(), Packet::new(Opcode::RecvMove, fen.to_string()));
                                Packet::new(Opcode::SendMoveResp, fen)
                            },
                            Err(_) => Packet::error(Err::IllegalMove)
                        }
                    }
                    None => Packet::error(Err::GameDoesntExist)
                }
            }
            None => Packet::error(Err::GameDoesntExist)
        }
    }

    // get the id out of the packet to use as game id
    pub fn get_game_id(&mut self, packet: Packet) -> Result<u32, String> {
        let pl = packet.payload();
        if pl.len() != 4 {
            return Err("Malformed packet".to_string())
        }
        Ok(u32::from_be_bytes(pl.try_into().unwrap()))
    }

    pub fn leave_game(&mut self, user: &String) -> Packet {
        info!("{}: leave_game", user);
        if self.game_users.contains_key(user) {
            let id = self.game_users.remove(user).unwrap();
            Packet::new(Opcode::LeaveGameResp, id.to_string())
        } else {
            Packet::error(Err::GameDoesntExist)
        }
    }
}