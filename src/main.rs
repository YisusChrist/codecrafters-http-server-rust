use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut tcp_stream) => {
                // Define the HTTP response
                let response = "HTTP/1.1 200 OK\r\n\r\n";

                // Write the response to the TCP stream
                if let Err(err) = tcp_stream.write_all(response.as_bytes()) {
                    eprintln!("Error writing to stream: {}", err);
                }

                // Read and ignore data from the connection (you can process it later)
                let mut buffer = [0; 1024];
                while let Ok(n) = tcp_stream.read(&mut buffer) {
                    if n == 0 {
                        break;
                    }
                }

                println!("accepted new connection and sent 200 OK response");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
