use std::fs;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};
use std::thread;
use std::time::Duration;
use rustwebhello::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let thread_pool = ThreadPool::new(4);


    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread_pool.execute(|| {
            handle_connection(stream)
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    println!("Incoming request: {}", String::from_utf8_lossy(&buffer[..]));

    let get_root = b"GET / HTTP/1.1\r\n";
    let get_sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, content_path) = if buffer.starts_with(get_root) {
        ("HTTP/1.1 200 OK", "index.html")
    } else if buffer.starts_with(get_sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "index.html")    
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };
    let content = fs::read_to_string(content_path).unwrap();

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        content.len(),
        content
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
