use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::{fs, thread};

use rustyserver::ThreadPool;

static IP_ADDRESS: &'static str = "127.0.0.1";
static PORT: &'static str = "8080";

fn main() {
    // Bind to a specific IP address and port to listen for incoming connections
    let address = format!("{}:{}", IP_ADDRESS, PORT);
    let listener = TcpListener::bind(address).unwrap();

    // Create a thread pool to handle concurrent connections efficiently
    let pool = ThreadPool::new(4); // Create a pool with 4 threads

    // Continuously accept and handle new connections
    for stream in listener.incoming() {
        let stream = stream.unwrap(); // Wait for the next incoming connection

        // Move ownership of the stream to the closure for handling within the thread pool
        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

// Response from web server must follow a specific format:
//  * Status line (e.g., HTTP/1.1 200 OK)
//  * Headers (e.g., Content-Length)
//  * Empty line
//  * Message body (optional)
fn handle_connection(mut stream: TcpStream) {
    // Buffer to hold received data from the connection
    let mut buffer = [0; 1024];

    // Read data from the stream into the buffer
    stream.read(&mut buffer).unwrap();

    // Print the received request for debugging purposes (optional)
    // println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

    // Define byte sequences to identify different request types
    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    // Determine the appropriate response based on the request type
    let (status_line, filename) = if buffer.starts_with(get) {
        // Serve the index.html page for a regular GET request
        ("HTTP/1.1 200 OK", "public/index.html")
    } else if buffer.starts_with(sleep) {
        // Simulate a delay of 5 seconds for the sleep request
        thread::sleep(Duration::from_secs(5));
        // Then serve the index.html page
        ("HTTP/1.1 200 OK", "public/index.html")
    } else {
        // Return a 404 Not Found response for unrecognized requests
        ("HTTP/1.1 404 NOT FOUND", "public/404.html")
    };

    // Read the requested file content
    let contents = fs::read_to_string(filename).unwrap();

    // Format the response following the HTTP protocol
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    // Send the response back to the client
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
