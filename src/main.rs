use std::fs::OpenOptions;
use std::io::{self, Write, Read, BufWriter};
use std::net::TcpStream;
use chrono::Local; // Husk at tilføje chrono i Cargo.toml

fn log_message(message: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let log_entry = format!("{} - {}\n", timestamp, message);

    // Åbn eller opret logfilen i tilføjelsesmode
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("messages.log")
        .expect("Could not open log file");

    let mut writer = BufWriter::new(file);
    writer.write_all(log_entry.as_bytes()).expect("Failed to write to log file");
}

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

        // Log beskeden før den sendes
        log_message(&input.trim());

        stream.write_all(input.as_bytes()).expect("Failed to send message");

        let mut buffer = [0; 512];
        let bytes_read = stream.read(&mut buffer).expect("Failed to read response");
        let response = String::from_utf8_lossy(&buffer[..bytes_read]);

        println!("Server response: {}", response);
    }
}