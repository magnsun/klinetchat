use chrono::Local;
use serialport::SerialPort;
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

// Logger en besked med timestamp til en logfil
fn log_message(message: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let log_entry = format!("{} - {}\n", timestamp, message);
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("messages.log")
        .expect("Kunne ikke åbne logfilen");
    let mut writer = BufWriter::new(file);
    writer
        .write_all(log_entry.as_bytes())
        .expect("Kunne ikke skrive til logfilen");

}
// Funktion til at sende data til micro:bit
fn send_to_microbit(port: &mut Box<dyn SerialPort>, data: &str) {
    let bytes = data.as_bytes();
    match port.write_all(bytes) {
        Ok(_) => {
            println!("Sendt til micro:bit: {}", data);
        }
        Err(e) => {
            eprintln!("Fejl ved sending til micro:bit: {:?}", e);
        }
    }
}

fn main() {
    let port_name = "COM9"; // Skift til den korrekte port hvis nødvendigt
    let baud_rate = 115200;

    let mut port = serialport::new(port_name, baud_rate)

        .timeout(Duration::from_millis(1000))

        .open()

        .expect("Kunne ikke åbne den serielle port");

    let mut stream =

        TcpStream::connect("127.0.0.1:8080").expect("Kunne ikke oprette forbindelse til server");

    println!("Forbundet til server! Skriv beskeder eller 'exit' for at afslutte.");
    let (tx, rx) = mpsc::channel();
    let _stream_clone = stream.try_clone().expect("Kunne ikke klone TCP stream");
    // Tråd til at læse fra stdin
    thread::spawn(move || {
        let stdin = io::stdin();

        loop {
            let mut input = String::new();
            print!("Skriv besked: ");
            io::stdout().flush().unwrap();
            stdin.read_line(&mut input).expect("Kunne ikke læse input");
            if input.trim().to_lowercase() == "exit" {
                break;
            }
            tx.send(input.trim().to_string())
                .expect("Kunne ikke sende input til hovedtråden");
        }
    });
    let mut buffer: Vec<u8> = vec![0; 512];


    loop {
        // Læs fra micro:bit
        match port.read(buffer.as_mut_slice()) {
            Ok(t) if t > 0 => {
                let received_data = String::from_utf8_lossy(&buffer[..t]);
                println!("Modtaget fra micro:bit: {}", received_data);
                log_message(&received_data);
                stream
                    .write_all(received_data.as_bytes())
                    .expect("Kunne ikke sende til server");
            }

            Ok(_) => {} // Ingen data
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {} // Normal timeout
            Err(e) => {
                eprintln!("Fejl ved læsning fra micro:bit: {:?}", e);

            }

        }

        // Håndter input fra brugeren

        if let Ok(input) = rx.try_recv() {
            log_message(&input);
            stream
                .write_all(input.as_bytes())
                .expect("Kunne ikke sende besked til server");
            send_to_microbit(&mut port, &input); // Send til micro:bit


            // Læs svar fra server
            let mut response_buffer = [0; 512];
            match stream.read(&mut response_buffer) {
                Ok(bytes_read) => {
                    if bytes_read > 0 {
                        let response =
                            String::from_utf8_lossy(&response_buffer[..bytes_read]).to_string();
                        println!("Svar fra server: {}", response);
                    }
                }
                Err(e) => {
                    eprintln!("Fejl ved læsning fra server: {:?}", e);
                }
            }
        }
        thread::sleep(Duration::from_millis(1000)); // Undgå at bruge for mange CPU-ressourcer
    }
}