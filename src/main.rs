use std::fs::File;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let directory = if args.len() >= 3 && args[1] == "--directory" {
        &args[2]
    } else {
        // Default directory when --directory is not provided
        "/path/to/default/directory"
    };

    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221")?;
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
    Ok(())
}

fn handle_client(stream: TcpStream, directory: &str) {
    let request = read_request(&stream);

    match request {
        Ok((request_str, headers, body)) => {
            println!("Received HTTP request:\n{}", request_str);
            process_request(&stream, &request_str, &headers, &body, directory);
        }
        Err(err) => {
            eprintln!("Error reading request: {}", err);
        }
    }

    if let Err(err) = stream.shutdown(std::net::Shutdown::Both) {
        eprintln!("Error shutting down stream: {}", err);
    }
}

fn read_request(mut stream: &TcpStream) -> io::Result<(String, Vec<String>, String)> {
    let mut request = String::new();
    let headers;

    // Read the request into a string
    stream.read_to_string(&mut request)?;

    // Split the request into headers and body
    let (header_section, body) = if let Some(end) = request.find("\r\n\r\n") {
        let body_start = end + 4;
        let (header_section, body) = request.split_at(body_start);
        (header_section, body)
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Incomplete header",
        ));
    };

    // Extract headers
    headers = header_section.lines().map(String::from).collect();

    Ok((header_section.to_string(), headers, body.to_string()))
}


fn process_request(
    mut stream: &TcpStream,
    request_str: &str,
    headers: &[String],
    body: &str,
    directory: &str,
) {
    let path = match extract_path(&request_str) {
        Some(p) => p,
        None => {
            eprintln!("Invalid request format");
            return;
        }
    };

    println!("Extracted path: {:?}", path);

    if path == "/" {
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
        if let Some(user_agent) = headers
            .iter()
            .find(|header| header.starts_with("User-Agent: "))
        {
            let user_agent_value = user_agent.replace("User-Agent: ", "");
            println!("User-Agent: {:?}", user_agent_value);

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                user_agent_value.len(),
                user_agent_value
            );
            send_response(stream, &response);
        }
    } else if let Some(filename) = extract_filename(&path) {
        let file_path = format!("{}/{}", directory, filename);

        println!(
            "Received {} request",
            request_str.split(' ').next().unwrap()
        );

        if request_str.starts_with("POST") {
            let response: &str;

            println!("Received file contents:\n{}", body);

            println!("Saved file to: {}", file_path);
            if let Err(err) = save_file(&file_path, body) {
                eprintln!("Error saving file: {}", err);
                response = "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\n\r\n";
            } else {
                println!("File saved successfully");
                response = "HTTP/1.1 201 Created\r\nContent-Length: 0\r\n\r\n";
            }
            send_response(stream, &response);
        } else {
            if let Ok(mut file) = File::open(&file_path) {
                let mut file_contents = Vec::new();
                if let Err(err) = file.read_to_end(&mut file_contents) {
                    eprintln!("Error reading file: {}", err);
                    let response =
                        "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\n\r\n";
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
                let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
                send_response(stream, response);
            }
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

fn save_file(file_path: &str, contents: &str) -> io::Result<()> {
    let mut file = File::create(file_path)?;
    file.write_all(contents.as_bytes())?;
    Ok(())
}

fn extract_random_string(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 2 && parts[1] == "echo" {
        Some(parts[2..].join("/").to_string())
    } else {
        None
    }
}
