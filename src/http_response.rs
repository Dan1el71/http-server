use core::fmt;
use std::collections::HashMap;

enum StatusCode {
    OK,
    Created,
    NotFound,
    MethodNotAllowed,
    BadRequest,
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let status_code = match self {
            StatusCode::OK => "200 OK",
            StatusCode::Created => "201 Created",
            StatusCode::BadRequest => "400 Bad Request",
            StatusCode::NotFound => "404 Not Found",
            StatusCode::MethodNotAllowed => "405 Method Not Allowed",
        };

        write!(f, "{status_code}")
    }
}
pub struct HttpResponse {
    status: StatusCode,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl From<StatusCode> for HttpResponse {
    fn from(status: StatusCode) -> Self {
        HttpResponse {
            status,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }
}

impl HttpResponse {
    pub fn new() -> Self {
        HttpResponse::from(StatusCode::OK)
    }

    pub fn ok(header: HashMap<String, String>, body: Vec<u8>) -> Self {
        HttpResponse::from((StatusCode::OK, header, body))
    }

    pub fn created() -> Self {
        HttpResponse::from(StatusCode::Created)
    }

    pub fn not_found() -> Self {
        HttpResponse::from(StatusCode::NotFound)
    }

    pub fn bad_request() -> Self {
        HttpResponse::from(StatusCode::BadRequest)
    }

    pub fn method_not_allowed() -> Self {
        HttpResponse::from(StatusCode::MethodNotAllowed)
    }
}
impl From<(StatusCode, HashMap<String, String>, Vec<u8>)> for HttpResponse {
    fn from(response: (StatusCode, HashMap<String, String>, Vec<u8>)) -> Self {
        HttpResponse {
            status: response.0,
            headers: response.1,
            body: response.2,
        }
    }
}

impl HttpResponse {
    pub fn to_string(&self) -> String {
        let mut response = format!("HTTP/1.1 {}\r\n", self.status);

        for (key, value) in &self.headers {
            response.push_str(&format!("{key}: {value}\r\n"));
        }

        response.push_str("\r\n");

        let body = String::from_utf8_lossy(&self.body);
        response.push_str(&body);

        response
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}
