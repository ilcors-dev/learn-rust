use core::panic;
use std::{
    collections::HashMap,
    fmt::{self, Display},
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
    str::FromStr,
};

#[derive(Debug)]
pub enum Method {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
    Patch,
}

impl FromStr for Method {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Method::Get),
            "HEAD" => Ok(Method::Head),
            "POST" => Ok(Method::Post),
            "PUT" => Ok(Method::Put),
            "DELETE" => Ok(Method::Delete),
            "CONNECT" => Ok(Method::Connect),
            "OPTIONS" => Ok(Method::Options),
            "TRACE" => Ok(Method::Trace),
            "PATCH" => Ok(Method::Patch),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct HttpRequest {
    pub protocol: String,
    pub method: Method,
    pub uri: String,
    pub headers: Vec<String>,
    pub body: String,
}

#[derive(Debug)]
pub struct HttpResponse {
    pub protocol: String,
    pub status_code: u16,
    pub reason_phrase: String,
    pub headers: Vec<String>,
    pub body: Option<String>,
}

#[derive(Debug)]
pub struct Http {
    handlers: HashMap<String, fn(&HttpRequest) -> HttpResponse>,
}

impl Http {
    pub fn new() -> Http {
        Http {
            handlers: HashMap::new(),
        }
    }

    pub fn register_handler(
        &mut self,
        uri: String,
        f: fn(request: &HttpRequest) -> HttpResponse,
    ) -> &Http {
        self.handlers.insert(uri, f);

        self
    }

    pub fn handle(&self, request: &HttpRequest, stream: &mut TcpStream) {
        let action = self.handlers.get(&request.uri);

        if action.is_none() {
            stream
                .write_all(
                    response(request, 404, "Not Found".to_string(), &vec![], None)
                        .to_string()
                        .as_bytes(),
                )
                .expect("Error sending reply");

            return;
        }

        let r = action.unwrap()(request);

        stream
            .write_all(r.to_string().as_bytes())
            .expect("Error sending reply");
    }
}

impl Display for HttpResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut response_string = String::new();

        // start line
        response_string.push_str(&self.protocol);
        response_string.push(' ');
        response_string.push_str(&self.status_code.to_string());
        response_string.push(' ');
        response_string.push_str(&self.reason_phrase);
        response_string.push_str("\r\n");

        // headers
        for header in &self.headers {
            response_string.push_str(header);
            response_string.push_str("\r\n");
        }
        response_string.push_str("\r\n");

        // body
        if let Some(body) = &self.body {
            response_string.push_str(body);
        }

        write!(f, "{}", response_string)
    }
}

pub fn http_parse(stream: &TcpStream) -> HttpRequest {
    let mut reader = BufReader::new(stream);

    let mut request = HttpRequest {
        protocol: "".to_string(),
        method: Method::Get,
        uri: "".to_string(),
        headers: vec![],
        body: "".to_string(),
    };

    let mut headers_ok = false;
    let mut content_length = 0;

    for (i, line) in reader.by_ref().lines().enumerate() {
        let line = match line {
            Ok(s) => s,
            Err(e) => panic!("Error reading http request line! {:?}", e),
        };

        if line.is_empty() {
            headers_ok = true;
            break;
        }

        if i == 0 {
            let parts: Vec<&str> = line.split_whitespace().collect();
            request.method = Method::from_str(parts[0]).expect("Error decoding request method");
            request.uri = parts[1].to_string();
            request.protocol = parts[2].to_string();
        } else {
            if line.to_lowercase().starts_with("content-length:") {
                let parts: Vec<&str> = line.split(": ").collect();
                content_length = parts[1].parse().unwrap_or(0);
            }
            request.headers.push(line);
        }
    }

    if headers_ok && content_length > 0 {
        let mut body = vec![0; content_length];

        reader.read_exact(&mut body).expect("Error reading body");

        request.body = String::from_utf8(body).expect("Error reading http body");
    }

    request
}

// pub fn handle(uri: &String, f: fn(request: &HttpRequest) -> HttpResponse) -> HttpResponse {}

pub fn response(
    request: &HttpRequest,
    status_code: u16,
    reason_phrase: String,
    headers: &Vec<String>,
    body: Option<String>,
) -> HttpResponse {
    HttpResponse {
        protocol: request.protocol.clone(),
        status_code,
        reason_phrase,
        headers: headers.clone(),
        body,
    }
}
