use std::fmt::Display;

use chess::{Board, ChessMove};

#[derive(Clone, Debug)]
pub struct Game {
    board: Board,
    p1: Option<String>,
    p2: Option<String>,
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
    pub fn new(p: String, white:bool) -> Self {
        if white {
            Game {
                board: Board::default(),
                p1: Some(p),
                p2: None,
                status: GameStatus::Open
            }
        } else {
            Game {
                board: Board::default(),
                p2: Some(p),
                p1: None,
                status: GameStatus::Open
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

    pub fn make_move(&mut self, mov: String) {
        match ChessMove::from_san(&self.board, &mov) {
            Ok(m) => self.board.to_owned().make_move(m, &mut self.board),
            Err(e) => {println!("Error: {}", e)}
        }
    }

    pub fn players(&self) -> Vec<String> {
        let mut ps: Vec<String> = Vec::new();
        match &self.p1 {
            Some(p) => {
                ps.push(p.to_string());
            }
            None => {}
        }
        match &self.p2 {
            Some(p) => {
                ps.push(p.to_string());
            }
            None => {}
        }
        ps
    }
}