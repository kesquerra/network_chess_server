use std::{net::{TcpListener, TcpStream, Shutdown}, io::Read};
use server::Server;
mod game;
mod server;
use env_logger::Env;
use std::time::Duration;


fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let mut server = Server::new();
    let listener = TcpListener::bind("127.0.0.1:8088")?;
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                match server.process_stream(&s) {
                    Ok(_) => {s.shutdown(Shutdown::Both);},
                    Err(e) => {
                        s.shutdown(Shutdown::Both);
                    }
                }
            }
            Err(e) => {
                println!("Error: {}", e.to_string());
            }
        }
        
    }
    Ok(())
}






