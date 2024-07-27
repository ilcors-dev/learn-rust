use std::{fs::File, io::Read, net::TcpListener};

use http::{HttpRequest, HttpResponse};

use crate::http::Http;

mod http;

fn main() {
    let s = TcpListener::bind("127.0.0.1:9999");

    let socket = match s {
        Ok(s) => s,
        Err(_e) => panic!("Could not bind listener"),
    };

    println!("Listening to address localhost:9999");

    let mut http = Http::new("/Users/ilcors-dev/src/learn-rust/http".to_string());

    http.register_handler('/'.to_string(), handle_root);
    http.register_handler("/index".to_string(), handle_index);

    for stream in socket.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(e) => panic!("{:?}", e),
        };

        println!("Accepted request");

        let request = http::http_parse(&stream);
        http.handle(&request, &mut stream);
    }
}

fn handle_root(request: &HttpRequest) -> HttpResponse {
    http::response(request, 200, "OK".to_string(), &vec![], None)
}

fn handle_index(request: &HttpRequest) -> HttpResponse {
    let mut file = File::open("./static/index.html").expect("Error opening file");

    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Error reading static file");

    http::response(
        request,
        200,
        "OK".to_string(),
        &vec![],
        Some(content.to_string()),
    )
}
