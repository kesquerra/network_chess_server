use std::fmt::Display;

use chess::{Board, ChessMove, Color};
use chess_lib::packet::Packet;
use tokio::sync::broadcast::Sender;

#[derive(Clone, Debug)]
pub struct Game {
    board: Board,
    p1: Option<String>,
    p2: Option<String>,
    p1c: Option<Sender<Packet>>,
    p2c: Option<Sender<Packet>>,
    status: GameStatus
}

#[derive(Clone, Debug)]
pub enum GameStatus {
    Open,
    Ongoing,
    Draw,
    WhiteWon,
    BlackWon
}

impl Display for GameStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Open => "open",
            Ongoing => "closed",
            Draw => "1/2-1/2",
            WhiteWon => "1-0",
            BlackWon => "0-1"
        };
        write!(f, "{}", str)
    }
}

impl Game {
    pub fn new(p: String, pc: Sender<Packet>, white:bool) -> Self {
        if white {
            Game {
                board: Board::default(),
                p1: Some(p),
                p1c: Some(pc),
                p2: None,
                p2c: None,
                status: GameStatus::Open
            }
        } else {
            Game {
                board: Board::default(),
                p2: Some(p),
                p2c: Some(pc),
                p1: None,
                p1c: None,
                status: GameStatus::Open
            }
        }
        
    }

    pub fn fen(&self) -> String {
        format!("{}", self.board)
    }

    pub fn is_white_turn(&self) -> bool {
        self.board.side_to_move() == Color::White
    }

    pub fn send(&mut self, white:bool, pkt: Packet) {
        if white {
            match &self.p1c {
                Some(v) => {v.send(pkt).unwrap();}
                None => {}
            }
        } else {
            match &self.p2c {
                Some(v) => {v.send(pkt).unwrap();}
                None => {}
            }
        }
    } 

    pub fn join(&mut self, p: String) -> Result<(), String> {
        match (&self.p1, &self.p2) {
            (Some(_), Some(_)) => Err("Game is full".to_string()),
            (Some(_), None) => {
                self.p2 = Some(p);
                Ok(())
            },
            (None, Some(_)) => {
                self.p1 = Some(p);
                Ok(())
            }
            (None, None) => {
                Err("Empty game".to_string())
            }
        }
    }

    pub fn status(&self) -> GameStatus {
        self.status.clone()
    }

    pub fn make_move(&mut self, mov: String) -> Result<String, String> {
        match ChessMove::from_san(&self.board, &mov) {
            Ok(m) => {
                if self.board.legal(m) {
                    self.board.to_owned().make_move(m, &mut self.board);
                    Ok(format!("{}", self.board))
                } else {
                    Err("Illegal move".to_string())
                }
            },
            Err(e) => Err(e.to_string())
        }
    }
}