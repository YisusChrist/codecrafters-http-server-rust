use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_client(mut stream: TcpStream) {
    let mut request = Vec::new();
    let mut buffer = [0; 1024];
    
    // Limit the read operation to a reasonable number of bytes
    while let Ok(n) = stream.read(&mut buffer) {
        if n == 0 {
            break;
        }
        request.extend_from_slice(&buffer[..n]);

        // Check if we've received a complete HTTP request (ending with \r\n\r\n)
        if request.ends_with(b"\r\n\r\n") {
            break;
        }
    }

    // Convert the request bytes to a string for processing
    let request_str = match String::from_utf8(request) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error parsing request: {}", e);
            return;
        }
    };

    println!("Received HTTP request:\n{}", request_str);

    // Extract the path from the request
    let path = match extract_path(&request_str) {
        Some(p) => p,
        None => {
            eprintln!("Invalid request format");
            return;
        }
    };

    // Define the HTTP response based on the path
    let response = if path == "/" {
        "HTTP/1.1 200 OK\r\n\r\n"
    } else {
        "HTTP/1.1 404 Not Found\r\n\r\n"
    };

    // Write the response to the TCP stream
    if let Err(err) = stream.write_all(response.as_bytes()) {
        eprintln!("Error writing response: {}", err);
    } else {
        println!("Sent response:\n{}", response);
    }

    // Close the TCP stream to signal the end of the response
    if let Err(err) = stream.shutdown(std::net::Shutdown::Both) {
        eprintln!("Error shutting down stream: {}", err);
    }
}

fn extract_path(request: &str) -> Option<&str> {
    let start = request.find(' ')? + 1;
    let end = request[start..].find(' ')? + start;
    Some(&request[start..end])
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
