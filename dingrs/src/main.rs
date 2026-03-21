use std::io::{Read, Write};
use std::net::TcpStream;

const PORT: u16 = 6666;
const IP: &str = "127.0.0.1";
const NETUPSRV: &str = "{IP}:{PORT}";

struct Client {
    stream: TcpStream,
}

impl Client {
    fn init() -> Self {
        let mut stream = TcpStream::connect(NETUPSRV).unwrap();
        return Self { stream };
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let command = &args[1];
    let arg1 = &args[2];
    let arg2 = &args[3];

    match command.as_str() {
        "ping" => {
            println!("CLI: Beginning ping..");
            let client = Client::init();
            println!("CLI: Completed ping..");
        }
        _ => panic!("Unknown command: {command}"),
    }
}
