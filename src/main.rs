use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_client(stream: TcpStream, directory: &str) {
    // Test the handling of multiple concurrent connections
    let request = read_request(&stream);

    match request {
        Ok((request_str, headers)) => {
            println!("Received HTTP request:\n{}", request_str);
            process_request(&stream, &request_str, &headers, directory);
        }
        Err(err) => {
            eprintln!("Error reading request: {}", err);
        }
    }

    if let Err(err) = stream.shutdown(std::net::Shutdown::Both) {
        eprintln!("Error shutting down stream: {}", err);
    }
}

fn read_request(mut stream: &TcpStream) -> Result<(String, Vec<String>), std::io::Error> {
    let mut request = String::new();
    let mut headers = Vec::new();
    let mut buffer = [0; 1024];
    let mut header_complete = false;

    while let Ok(n) = stream.read(&mut buffer) {
        if n == 0 {
            break;
        }
        request.push_str(&String::from_utf8_lossy(&buffer[..n]));

        if !header_complete {
            if let Some(end) = request.find("\r\n\r\n") {
                let header_section = &request[..=end];
                headers = header_section.lines().map(|s| s.to_string()).collect();
                header_complete = true;
            }
        }

        if request.ends_with("\r\n\r\n") {
            break;
        }
    }

    if header_complete {
        Ok((request, headers))
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Incomplete header",
        ))
    }
}

fn process_request(mut stream: &TcpStream, request_str: &str, headers: &[String], directory: &str) {
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
    } else if path == "/user-agent" {
        // Find the User-Agent header value
        let user_agent = headers
            .iter()
            .find(|header| header.starts_with("User-Agent: "));
        if let Some(user_agent) = user_agent {
            let user_agent_value = user_agent.replace("User-Agent: ", "");
            println!("User-Agent: {:?}", user_agent_value);

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                user_agent_value.len(),
                user_agent_value
            );
            send_response(stream, &response);
            return;
        }
    } else if let Some(filename) = extract_filename(&path) {
        let file_path = format!("{}/{}", directory, filename);

        if let Ok(mut file) = File::open(&file_path) {
            let mut file_contents = Vec::new();
            if let Err(err) = file.read_to_end(&mut file_contents) {
                eprintln!("Error reading file: {}", err);
                // Respond with a 500 Internal Server Error if reading the file fails
                let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\n\r\n";
                send_response(stream, response);
                return;
            }

            let content_type = "application/octet-stream";
            let content_length = file_contents.len();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
                content_type, content_length
            );

            send_response(stream, &response);
            if let Err(err) = stream.write_all(&file_contents) {
                eprintln!("Error writing file contents: {}", err);
            }
        } else {
            // Respond with a 404 Not Found if the file doesn't exist
            let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
            send_response(stream, response);
        }
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

fn extract_filename(path: &str) -> Option<&str> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() == 3 && parts[1] == "files" {
        Some(parts[2])
    } else {
        None
    }
}

fn extract_random_string(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 2 && parts[1] == "echo" {
        Some(parts[2..].join("/").to_string())
    } else {
        None
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 || args[1] != "--directory" {
        eprintln!("Usage: {} --directory <directory>", args[0]);
        return;
    }

    let directory = &args[2];
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(tcp_stream) => {
                println!("Accepted new connection");

                let directory = directory.to_string();
                thread::spawn(move || {
                    handle_client(tcp_stream, &directory);
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
