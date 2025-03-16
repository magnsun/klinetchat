use std::io::{self, Write, Read};
use std::net::TcpStream;

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8080").expect("Could not connect to server");
    println!("Connected to server! Type messages or '/message' to test response.");

    loop {
        let mut input = String::new();
        print!("Enter message: ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        if input.trim().to_lowercase() == "exit" {
            println!("Disconnecting...");
            break;
        }

        stream.write_all(input.as_bytes()).expect("Failed to send message");

        let mut buffer = [0; 512];
        let bytes_read = stream.read(&mut buffer).expect("Failed to read response");
        let response = String::from_utf8_lossy(&buffer[..bytes_read]);

        println!("Server response: {}", response);
    }
}