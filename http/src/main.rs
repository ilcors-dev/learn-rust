use std::{
    env,
    fs::File,
    io::Read,
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
};

use http::{HttpRequest, HttpResponse};

use crate::http::Http;

mod http;

struct Config<'a> {
    /// The base path where the server will start looking for static files requested by the clients
    resources_base_path: &'a str,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut config = Config {
        resources_base_path: "/Users/ilcors-dev/src/learn-rust/http",
    };

    if args.len() > 1 && !args[1].is_empty() {
        config.resources_base_path = &args[1];
    }

    let s = TcpListener::bind("0.0.0.0:9999");

    let socket = match s {
        Ok(s) => s,
        Err(_e) => panic!("Could not bind listener"),
    };

    println!("Listening to address localhost:9999");

    let http = Arc::new(Mutex::new(Http::new(
        "/Users/ilcors-dev/src/learn-rust/http".to_string(),
    )));

    http.lock()
        .unwrap()
        .register_handler('/'.to_string(), handle_root);
    http.lock()
        .unwrap()
        .register_handler("/index".to_string(), handle_index);

    for stream in socket.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(e) => panic!("{:?}", e),
        };

        let http = Arc::clone(&http);

        thread::spawn(move || {
            let ip = stream.local_addr().expect("Failed to retrive address");

            println!("Accepted request from address {}", ip);

            let request = http::http_parse(&stream);
            let http = http.lock().unwrap();
            http.handle(&request, &mut stream);
        });
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
