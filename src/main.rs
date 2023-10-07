use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_client(stream: TcpStream) {
    let request = read_request(&stream);

    match request {
        Ok(request_str) => {
            println!("Received HTTP request:\n{}", request_str);
            process_request(&stream, &request_str);
        }
        Err(err) => {
            eprintln!("Error reading request: {}", err);
        }
    }

    if let Err(err) = stream.shutdown(std::net::Shutdown::Both) {
        eprintln!("Error shutting down stream: {}", err);
    }
}

fn read_request(mut stream: &TcpStream) -> Result<String, std::io::Error> {
    let mut request = Vec::new();
    let mut buffer = [0; 1024];

    while let Ok(n) = stream.read(&mut buffer) {
        if n == 0 {
            break;
        }
        request.extend_from_slice(&buffer[..n]);

        if request.ends_with(b"\r\n\r\n") {
            break;
        }
    }

    String::from_utf8(request).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

fn process_request(stream: &TcpStream, request_str: &str) {
    let path = match extract_path(&request_str) {
        Some(p) => p,
        None => {
            eprintln!("Invalid request format");
            return;
        }
    };

    println!("Extracted path: {:?}", path);

    if path == "/" {
        // Respond with a 200 OK for the root path
        let response = "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
        send_response(stream, response);
    } else if let Some(random_string) = extract_random_string(&path) {
        println!("Extracted random string: {:?}", random_string);

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            random_string.len(),
            random_string
        );
        send_response(stream, &response);
    } else {
        let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
        send_response(stream, response);
    }
}

fn send_response(mut stream: &TcpStream, response: &str) {
    if let Err(err) = stream.write_all(response.as_bytes()) {
        eprintln!("Error writing response: {}", err);
    } else {
        println!("Sent response:\n{}", response);
    }
}

fn extract_path(request: &str) -> Option<&str> {
    let start = request.find(' ')? + 1;
    let end = request[start..].find(' ')? + start;
    Some(&request[start..end])
}

fn extract_random_string(path: &str) -> Option<&str> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() == 3 && parts[1] == "echo" {
        Some(parts[2])
    } else {
        None
    }
}

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(tcp_stream) => {
                println!("Accepted new connection");

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
