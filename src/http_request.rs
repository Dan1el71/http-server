use std::{
    collections::HashMap,
    env::args,
    fs::File,
    io::{Read, Result, Write},
    net::TcpStream,
    path::Path,
};

use nom::AsBytes;

use crate::http_response::HttpResponse;

enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

impl HttpMethod {
    fn from_str(method: &str) -> Self {
        match method {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "DELETE" => HttpMethod::DELETE,
            _ => panic!("Unknown method: {method}"),
        }
    }
}
struct HttpRequest {
    method: HttpMethod,
    path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl HttpRequest {
    fn new(request: &str) -> Self {
        let lines: Vec<&str> = request.split("\r\n").collect();
        let tokens: Vec<&str> = lines[0].split(' ').collect();

        let mut body = Vec::new();
        let mut in_body = false;

        let mut headers = HashMap::new();

        for line in lines.iter().skip(1) {
            if line.is_empty() {
                in_body = true;
                continue;
            }

            if in_body {
                body.extend_from_slice(line.as_bytes());
                body.push(b'\n');
            } else {
                let header: Vec<&str> = line.split(": ").collect();
                headers.insert(header[0].to_string(), header[1].to_string());
            }
        }

        HttpRequest {
            method: HttpMethod::from_str(tokens[0]),
            path: tokens[1].to_string(),
            headers,
            body,
        }
    }
}

fn read_file(file_path: &str) -> Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

fn save_file(dir: &str, file_name: &str, body: Vec<u8>) -> Result<()> {
    let file_path = Path::new(dir).join(file_name);
    create_file(&file_path, &body)
}

fn create_file(file_path: &Path, buf: &[u8]) -> Result<()> {
    let mut file = File::create(file_path)?;

    println!("Buffer: {:?}", String::from_utf8_lossy(buf));
    file.write_all(buf)?;

    file.flush()?;
    Ok(())
}

fn create_headers(content_type: &str, content_length: usize) -> HashMap<String, String> {
    let mut headers: HashMap<String, String> = HashMap::new();
    headers.insert("Content-Type".to_string(), content_type.to_string());
    headers.insert("Content-Length".to_string(), content_length.to_string());
    headers
}

fn handle_request(mut stream: TcpStream) -> Result<HttpRequest> {
    let mut buffer = [0u8; 1024];
    stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer);
    Ok(HttpRequest::new(&request))
}

fn handle_get_method(request: HttpRequest) -> HttpResponse {
    let request_path: String = request.path;

    match request_path.as_str() {
        "/" => HttpResponse::new(),
        path if path.starts_with("/echo/") => {
            let echo = &path[6..];
            let echo_as_bytes = echo.as_bytes();
            let headers: HashMap<String, String> =
                create_headers("text/plain", echo_as_bytes.len());

            HttpResponse::ok(headers, echo_as_bytes.to_vec())
        }
        "/user-agent" => {
            if let Some(user_agent) = request.headers.get("User-Agent") {
                let headers: HashMap<String, String> =
                    create_headers("text/plain", user_agent.len());
                HttpResponse::ok(headers, user_agent.as_bytes().to_vec())
            } else {
                HttpResponse::not_found()
            }
        }
        path if path.starts_with("/files/") => {
            if let Some(dir) = args().nth(2) {
                let file_path = &path[7..];

                match read_file(Path::new(&dir).join(file_path).to_str().unwrap()) {
                    Ok(buf) => {
                        let headers: HashMap<String, String> =
                            create_headers("application/octet-stream", buf.len());
                        HttpResponse::ok(headers, buf)
                    }
                    Err(_) => HttpResponse::not_found(),
                }
            } else {
                HttpResponse::not_found()
            }
        }
        _ => HttpResponse::not_found(),
    }
}

fn handle_post_method(request: HttpRequest) -> HttpResponse {
    let request_path = request.path;

    if request_path.starts_with("/files/") {
        if let Some(dir) = args().nth(2) {
            let file_name = &request_path[7..];
            let body = request.body;
            return match save_file(&dir, file_name, body) {
                Ok(()) => HttpResponse::created(),
                Err(e) => {
                    eprintln!("Failed to create file: {e}");
                    HttpResponse::bad_request()
                }
            };
        } else {
            return HttpResponse::not_found();
        }
    }

    HttpResponse::not_found()
}

pub fn handle_stream(mut stream: TcpStream) -> Result<()> {
    println!("Connection established");

    match handle_request(stream.try_clone()?) {
        Ok(request) => match request.method {
            HttpMethod::GET => {
                stream.write_all(handle_get_method(request).to_string().as_bytes())?;
            }
            HttpMethod::POST => {
                stream.write_all(handle_post_method(request).to_string().as_bytes())?;
            }
            _ => {
                stream.write_all(&HttpResponse::method_not_allowed().as_bytes())?;
            }
        },
        Err(e) => {
            eprintln!("Failed to read request: {e}");
            stream.write_all(&HttpResponse::bad_request().as_bytes())?;
        }
    }

    Ok(())
}
