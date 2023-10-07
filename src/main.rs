use std::fs::File;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

/*
struct HttpRequest {
    request_str: String,
    headers: Vec<String>,
    body: String,
}
*/
struct HttpResponse {
    status: &'static str,
    content_type: Option<&'static str>,
    content_length: Option<usize>,
    body: Option<String>,
}

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
    let mut headers = Vec::new();
    let mut buffer = [0; 1024];
    let mut header_complete = false;
    let mut content_length = 0;

    while let Ok(n) = stream.read(&mut buffer) {
        if n == 0 {
            break;
        }
        request.push_str(&String::from_utf8_lossy(&buffer[..n]));

        if !header_complete {
            if let Some(end) = request.find("\r\n\r\n") {
                let header_section = &request[..=end];
                headers = header_section.lines().map(String::from).collect();
                header_complete = true;

                if let Some(length_str) = headers.iter().find(|s| s.starts_with("Content-Length: "))
                {
                    let parts: Vec<&str> = length_str.splitn(2, ' ').collect();
                    if parts.len() == 2 {
                        if let Ok(length) = parts[1].parse::<usize>() {
                            content_length = length;
                        }
                    }
                }
            }
        }

        if header_complete && request.len() >= content_length {
            break;
        }
    }

    if header_complete {
        let body_start = request.find("\r\n\r\n").unwrap_or(0) + 4;
        let body = request.split_off(body_start);
        Ok((request, headers, body))
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Incomplete header",
        ))
    }
}

fn process_request(
    stream: &TcpStream,
    request_str: &str,
    headers: &[String],
    body: &str,
    directory: &str,
) {
    let response: HttpResponse = match extract_path(request_str) {
        Some("/") => HttpResponse {
            status: "HTTP/1.1 200 OK",
            content_type: None,
            content_length: Some(0),
            body: None,
        },
        Some(path) if path.starts_with("/echo/") => {
            let random_string = path.trim_start_matches("/echo/");
            HttpResponse {
                status: "HTTP/1.1 200 OK",
                content_type: Some("text/plain"),
                content_length: Some(random_string.len()),
                body: Some(random_string.to_string()),
            }
        }
        Some("/user-agent") => {
            if let Some(user_agent) = headers
                .iter()
                .find(|header| header.starts_with("User-Agent: "))
            {
                let user_agent_value = user_agent.replace("User-Agent: ", "");
                HttpResponse {
                    status: "HTTP/1.1 200 OK",
                    content_type: Some("text/plain"),
                    content_length: Some(user_agent_value.len()),
                    body: Some(user_agent_value),
                }
            } else {
                HttpResponse {
                    status: "HTTP/1.1 400 Bad Request",
                    content_type: None,
                    content_length: Some(0),
                    body: None,
                }
            }
        }
        Some(path) => {
            if let Some(filename) = extract_filename(path) {
                println!("Received filename: {}", filename);
                let file_path = format!("{}/{}", directory, filename);
                if request_str.starts_with("POST") {
                    handle_post_file_request(&file_path, body)
                } else {
                    handle_get_file_request(&file_path)
                }
            } else {
                HttpResponse {
                    status: "HTTP/1.1 404 Not Found",
                    content_type: None,
                    content_length: Some(0),
                    body: None,
                }
            }
        }
        None => HttpResponse {
            status: "HTTP/1.1 400 Bad Request",
            content_type: None,
            content_length: Some(0),
            body: None,
        },
    };

    send_response(stream, &response);
}

fn handle_get_file_request(file_path: &str) -> HttpResponse {
    if let Ok(mut file) = File::open(file_path) {
        let mut file_contents = Vec::new();
        if let Err(err) = file.read_to_end(&mut file_contents) {
            eprintln!("Error reading file: {}", err);
            return HttpResponse {
                status: "HTTP/1.1 500 Internal Server Error",
                content_type: None,
                content_length: Some(0),
                body: None,
            };
        }

        let content_type = "application/octet-stream";
        let content_length = file_contents.len();
        let body = String::from_utf8_lossy(&file_contents).into_owned();
        HttpResponse {
            status: "HTTP/1.1 200 OK",
            content_type: Some(content_type),
            content_length: Some(content_length),
            body: Some(body),
        }
    } else {
        HttpResponse {
            status: "HTTP/1.1 404 Not Found",
            content_type: None,
            content_length: Some(0),
            body: None,
        }
    }
}

fn handle_post_file_request(file_path: &str, body: &str) -> HttpResponse {
    println!("Received file contents:\n{}", body);

    if let Err(err) = save_file(file_path, body) {
        eprintln!("Error saving file: {}", err);
        HttpResponse {
            status: "HTTP/1.1 500 Internal Server Error",
            content_type: None,
            content_length: Some(0),
            body: None,
        }
    } else {
        println!("File saved successfully");

        let body = body.to_string();
        HttpResponse {
            status: "HTTP/1.1 201 Created",
            content_type: None,
            content_length: Some(0),
            body: Some(body),
        }
    }
}

fn send_response(mut stream: &TcpStream, response: &HttpResponse) {
    let mut response_str = response.status.to_string();
    if let Some(content_type) = response.content_type {
        response_str += &format!("\r\nContent-Type: {}", content_type);
    }
    if let Some(content_length) = response.content_length {
        response_str += &format!("\r\nContent-Length: {}", content_length);
    }
    response_str += "\r\n\r\n";

    if let Some(body) = &response.body {
        response_str += body;
    }

    if let Err(err) = stream.write_all(response_str.as_bytes()) {
        eprintln!("Error writing response: {}", err);
    } else {
        println!("Sent response:\n{}", response_str);
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
