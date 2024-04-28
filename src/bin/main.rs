use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::{fs, thread};

use rustyserver::ThreadPool;

static IP_ADDRESS: &'static str = "127.0.0.1";
static PORT: &'static str = "8080";

fn main() {
    let address = format!("{}:{}", IP_ADDRESS, PORT);

    // Create tcp listener
    let listener = TcpListener::bind(address).unwrap();

    // Create thread pool
    let pool = ThreadPool::new(4);

    // Loop through all connections
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

// Response from web server must be an following format:
/*
    HTTP-VERSION Status-Code Reason-Phrase CRLF
    headers CLRF
    message-body

    e.g: HTTP/1.1 200 OK\r\n\r\n
*/
fn handle_connection(mut stream: TcpStream) {
    // Create buffer to copy data from stream here
    let mut buffer = [0; 1024];
    // Read the stream anc copy data to buffer
    stream.read(&mut buffer).unwrap();

    // Print to console web server request
    //println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

    // Hardcoded get request header
    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        // Return index page
        ("HTTP/1.1 200 OK", "public/index.html")
    } else if buffer.starts_with(sleep) {
        // Return index page, but waiting 5 seconds
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "public/index.html")
    } else {
        // Return 404 page
        ("HTTP/1.1 404 NOT FOUND", "public/404.html")
    };

    // Return a response
    let contents = fs::read_to_string(filename).unwrap();
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
