use crate::{db, models::Mail};
use std::{
    io::{BufRead, BufReader, Write},
    net::{Shutdown, TcpStream},
};

const VERSION: &str = "0.1";
const SMTP_OK: &str = "250 OK\n";
const SMTP_START_MAIL_INPUT: &str = "354 Start mail input; end with <CRLF>.<CRLF>\n";
const SMTP_OK_QUEUED: &str = "250 OK: Queueds\n";
const SMTP_COMMAND_NOT_RECOGNIZED: &str = "500 Command not recognized\n";

pub fn handle(mut stream: TcpStream) {
    let mut mail = Mail {
        uid: uuid::Uuid::new_v4().to_string(),
        sender: String::new(),
        reciever: String::new(),
        body: String::new(),
    };

    let mut buffer = String::new();
    stream
        .write(format!("220 SMTP/{} READY\n", VERSION).as_bytes())
        .unwrap();
    // println!("SMTP Connection from: {}", stream.peer_addr().unwrap());
    loop {
        let mut reader = std::io::BufReader::new(&stream);
        match reader.read_line(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    break;
                }
                let mut parts = buffer.split_whitespace();
                let command = parts.next().map(|s| s.to_lowercase());
                let mut args = parts;
                match command.as_deref() {
                    Some("helo") | Some("ehlo") => {
                        let domain = args.next();
                        stream
                            .write(format!("250 Hello {}\n", domain.unwrap()).as_bytes())
                            .unwrap();
                    }
                    Some("mail") => {
                        let from = args.collect::<Vec<&str>>().join(" ");
                        if let Some(start) = from.find('<') {
                            if let Some(end) = from.find('>') {
                                mail.sender = from[start + 1..end].to_string();
                                stream.write(SMTP_OK.as_bytes()).unwrap();
                            }
                        }
                    }
                    Some("rcpt") => {
                        let to = args.collect::<Vec<&str>>().join(" ");
                        if let Some(start) = to.find('<') {
                            if let Some(end) = to.find('>') {
                                mail.reciever = to[start + 1..end].to_string();
                                stream.write(SMTP_OK.as_bytes()).unwrap();
                            }
                        }
                    }
                    Some("data") => {
                        drop(reader);
                        stream.write(SMTP_START_MAIL_INPUT.as_bytes()).unwrap();

                        let mut data_reader = BufReader::new(&stream);
                        let mut data_line = String::new();
                        while data_reader.read_line(&mut data_line).unwrap() > 0 {
                            if data_line == ".\r\n" {
                                break;
                            }
                            mail.body.push_str(&data_line);
                            data_line.clear();
                        }

                        stream
                            .write(SMTP_OK_QUEUED.as_bytes())
                            .expect("Failed to write to stream");
                        println!("Recieved Mail from: {} to {}", mail.sender, mail.reciever);
                        match db::add_mail(&mail) {
                            Ok(_) => {
                                println!("Mail added to database");
                            }
                            Err(e) => {
                                eprintln!("Error adding mail to database: {}", e);
                            }
                        }
                    }
                    Some("quit") => {
                        stream.write("221 Goodbye\n".as_bytes()).unwrap();
                        break;
                    }
                    Some("rset") => {
                        mail = Mail {
                            uid: uuid::Uuid::new_v4().to_string(),
                            sender: String::new(),
                            reciever: String::new(),
                            body: String::new(),
                        };
                        stream.write(SMTP_OK.as_bytes()).unwrap();
                    }
                    _ => {
                        println!("Command not recognized: {}", command.unwrap());
                        stream
                            .write(SMTP_COMMAND_NOT_RECOGNIZED.as_bytes())
                            .unwrap();
                    }
                }
                buffer.clear();
            }
            Err(e) => {
                eprintln!("Error reading from stream: {}", e);
                break;
            }
        }
    }
    stream.shutdown(Shutdown::Both).unwrap();
}
