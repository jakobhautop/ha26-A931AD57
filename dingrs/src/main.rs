use std::io::{Read, Write};
use std::net::TcpStream;

const NETUPSRV: &str = "127.0.0.1:6666";

struct Client {
    stream: TcpStream,
}

impl Client {
    fn init() -> Client {
        println!("INI: Beginning client init");
        println!("INI: Netupsrc on {}", NETUPSRV);
        let mut stream = TcpStream::connect(NETUPSRV).unwrap();
        println!("INI: Completed client init");
        return Self { stream };
    }

    fn send_and_recv(&mut self, b: &[u8]) {
        println!("SAR: Send {}", String::from_utf8_lossy(b));
        self.stream.write_all(b).unwrap();
        let mut buf = [0; 1024];
        let n = self.stream.read(&mut buf).unwrap();
        println!("SAR: Recv: {}", String::from_utf8_lossy(&buf[..n]));
    }

    fn ping(&mut self) {
        self.send_and_recv(b"Hi");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let command = &args[1];
    //let arg1 = &args[2];
    //let arg2 = &args[3];

    match command.as_str() {
        "ping" => {
            println!("CLI: Beginning ping..");
            let client = Client::init();
            println!("CLI: Completed ping..");
        }
        _ => panic!("Unknown command: {command}"),
    }
}
