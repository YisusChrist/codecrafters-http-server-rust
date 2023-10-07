use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_client(mut stream: TcpStream) {
    // Define the HTTP response
    let response = "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
    
    // Write the response to the TCP stream
    if let Err(err) = stream.write_all(response.as_bytes()) {
        eprintln!("Error writing to stream: {}", err);
    } else {
        println!("Sent 200 OK response");
    }

    // Read and ignore data from the connection (you can process it later)
    let mut buffer = [0; 1024];
    while let Ok(n) = stream.read(&mut buffer) {
        if n == 0 {
            break;
        }
    }

    println!("Finished processing connection");
}

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(tcp_stream) => {
                println!("Accepted new connection");

                // Spawn a new thread to handle the client
                thread::spawn(move || {
                    handle_client(tcp_stream);
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
