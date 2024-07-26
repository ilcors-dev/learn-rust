use core::panic;
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    str::FromStr,
};

#[derive(Debug)]
enum Method {
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
struct HttpRequest {
    protocol: String,
    method: Method,
    uri: String,
    headers: Vec<String>,
    body: String,
}

#[derive(Debug)]
struct HttpResponse {
    protocol: String,
    status_code: u16,
    reason_phrase: String,
    headers: Vec<String>,
    body: Option<String>,
}

impl HttpResponse {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];
        let mut s = String::new();

        // Start line
        s.push_str(&self.protocol);
        s.push(' ');
        s.push_str(&self.status_code.to_string());
        s.push(' ');
        s.push_str(&self.reason_phrase);
        s.push_str("\r\n");

        // Headers
        for header in &self.headers {
            s.push_str(header);
            s.push_str("\r\n");
        }
        s.push_str("\r\n"); // Blank line to indicate the end of headers

        // Write start line and headers to bytes
        bytes
            .write_all(s.as_bytes())
            .expect("Error serializing HttpResponse");

        // Body
        if let Some(body) = &self.body {
            bytes
                .write_all(body.as_bytes())
                .expect("Error serializing HttpResponse");
        }

        bytes
    }

    fn to_string(&self) -> String {
        let mut response_string = String::new();

        // Start line
        response_string.push_str(&self.protocol);
        response_string.push(' ');
        response_string.push_str(&self.status_code.to_string());
        response_string.push(' ');
        response_string.push_str(&self.reason_phrase);
        response_string.push_str("\r\n");

        // Headers
        for header in &self.headers {
            response_string.push_str(header);
            response_string.push_str("\r\n");
        }
        response_string.push_str("\r\n"); // Blank line to indicate the end of headers

        // Body
        if let Some(body) = &self.body {
            response_string.push_str(body);
        }

        response_string
    }
}

fn main() {
    let s = TcpListener::bind("127.0.0.1:9999");

    let socket = match s {
        Ok(s) => s,
        Err(_e) => panic!("Could not bind listener"),
    };

    println!("Listening to address localhost:9999");

    for stream in socket.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(e) => panic!("{:?}", e),
        };

        println!("Accepted request");

        let request = http_parse(&stream);

        println!("request: {:?}", request);

        let r = response(&request, 200, "OK".to_string(), &vec![], None);

        println!("sending response {:?}", r.to_string());

        stream
            .write_all(r.to_string().as_bytes())
            .expect("Error sending reply");
    }
}

fn http_parse(stream: &TcpStream) -> HttpRequest {
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

fn response(
    request: &HttpRequest,
    status_code: u16,
    reason_phrase: String,
    headers: &Vec<String>,
    body: Option<String>,
) -> HttpResponse {
    HttpResponse {
        protocol: request.protocol.clone(),
        status_code: status_code,
        reason_phrase: reason_phrase,
        headers: headers.clone(),
        body: body,
    }
}
