use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};


pub fn start_tcp_server(shared_file: Arc<Mutex<String>>) {
    let listener = TcpListener::bind("127.0.0.1:9000").expect("Failed to bind TCP server");
    println!("TCP Server running on 127.0.0.1:9000");

    for stream in listener.incoming() {
        let shared_file = shared_file.clone();
        thread::spawn(move || {
            let stream = stream.expect("Failed to accept connection");
            handle_client(stream, shared_file);
        });
    }
}

fn handle_client(mut stream: TcpStream, shared_file: Arc<Mutex<String>>) {
    let mut buffer = [0; 512];
    let bytes_read = stream.read(&mut buffer).expect("Failed to read from stream");
    let command = String::from_utf8_lossy(&buffer[..bytes_read]).trim().to_string();

    match command.as_str() {
        cmd if cmd.starts_with("write") => {
            let message = cmd.strip_prefix("write ").unwrap_or("");
            write_to_file(shared_file, message);
            stream.write_all(b"Write operation completed\n").expect("Failed to write to stream");
        }
        "read" => {
            let content = read_from_file(shared_file);
            stream.write_all(content.as_bytes()).expect("Failed to write to stream");
        }
        _ => {
            stream.write_all(b"Unknown command\n").expect("Failed to write to stream");
        }
    }
}

fn write_to_file(shared_file: Arc<Mutex<String>>, message: &str) {
    let mut file = shared_file.lock().unwrap();
    thread::sleep(Duration::from_secs(2)); // Simulate delay
    file.push_str(message);
    file.push('\n');
}

fn read_from_file(shared_file: Arc<Mutex<String>>) -> String {
    let file = shared_file.lock().unwrap();
    thread::sleep(Duration::from_secs(1)); // Simulate delay
    file.clone()
}
