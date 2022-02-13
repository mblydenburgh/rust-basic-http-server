use std::fs;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream)
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    println!("Incoming request: {}", String::from_utf8_lossy(&buffer[..]));

    let get_root = b"GET / HTTP/1.1\r\n";

    if buffer.starts_with(get_root) {
        let html_content = fs::read_to_string("index.html").unwrap();
        let status_line = "HTTP/1.1 200 OK";
        let response = format!(
            "{}\r\nContent-Length: {} \r\n\r\n{}",
            status_line,
            html_content.len(),
            html_content
        );

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    } else {
        // other requests
        let status_line = "HTTP/1.1 404 NOT FOUND";
        let html_content = fs::read_to_string("404.html").unwrap();
        let response = format!(
            "{}\r\nContent-Length: {}\r\n\r\n{}",
            status_line,
            html_content.len(),
            html_content
        );
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}
