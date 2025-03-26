use chrono::Local;
use serialport::SerialPort;
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use std::sync::mpsc;

fn log_message(message: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let log_entry = format!("{} - {}\n", timestamp, message);

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("messages.log")
        .expect("Could not open log file");

    let mut writer = BufWriter::new(file);
    writer.write_all(log_entry.as_bytes()).expect("Failed to write to log file");
}

fn main() {
    let port_name = "COM9"; // Ændr til det korrekte portnummer
    let baud_rate = 115200;

    // Forsøg på at åbne den serielle port
    let mut port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_secs(1))
        .open()
        .expect("Failed to open port");

    let mut stream = TcpStream::connect("127.0.0.1:8080").expect("Could not connect to server");
    println!("Connected to server! Type messages or '/message' to test response.");

    let (tx, rx) = mpsc::channel();
    let stream_clone = stream.try_clone().expect("Failed to clone TCP stream");

    // Thread for reading from stdin
    thread::spawn(move || {
        let stdin = io::stdin();
        loop {
            let mut input = String::new();
            print!("Enter message: ");
            io::stdout().flush().unwrap();
            stdin.read_line(&mut input).expect("Failed to read line");

            if input.trim().to_lowercase() == "exit" {
                break;
            }

            // Send the input through the channel to the main thread
            tx.send(input.trim().to_string()).expect("Failed to send input");
        }
    });

    let mut buffer: Vec<u8> = vec![0; 8]; // Buffer til data

    loop {
        // Check for serial port data
        match port.read(buffer.as_mut_slice()) {
            Ok(t) if t > 0 => {
                let received_data = String::from_utf8_lossy(&buffer[..t]);
                println!("Received from serial port: {}", received_data);

                // Log and send the received data to the server
                log_message(&received_data);
                stream.write_all(received_data.as_bytes()).expect("Failed to send data to server");
            }
            Ok(_) => {} // Ingen data læst
            Err(e) => {
                eprintln!("Error reading from port: {:?}", e);
            }
        }

        // Check for user input from the channel
        if let Ok(input) = rx.try_recv() {
            log_message(&input);
            stream.write_all(input.as_bytes()).expect("Failed to send message");

            let mut response_buffer = [0; 512];
            let bytes_read = stream.read(&mut response_buffer).expect("Failed to read response");
            let response = String::from_utf8_lossy(&response_buffer[..bytes_read]);

            println!("Server response: {}", response);
        }

        thread::sleep(Duration::from_millis(5000)); // Small delay to prevent busy waiting
    }
}