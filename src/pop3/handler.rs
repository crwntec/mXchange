use super::session;
use crate::{auth, db::{self, delete_mail}};
use base64::prelude::*;
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
    time::SystemTime,
};

const KEY: &str = "mx24.144";

pub fn handle(mut stream: TcpStream) {
    stream.write(b"+OK POP3 server ready\r\n").unwrap();
    let mut session = session::Session {
        user: None,
        password: None,
    };

    loop {
        let mut buffer = vec![0;1024];
        let bytes_read = {
            let mut reader = BufReader::new(&stream);
            match reader.read(&mut buffer) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        break;
                    }
                    bytes_read
                }
                Err(e) => {
                    
                    println!("Error while reading command: {}", e);
                    break;
                }
            }
        };
        let mut buffer = match String::from_utf8(buffer[..bytes_read].to_vec()) {
            Ok(buffer) => buffer,
            Err(_) => {
                println!("Recieved non UTF-8 message: ignoring");
                continue;
            },
        };
        let mut parts = buffer.split_whitespace();
        let command = parts.next();
        let mut args = parts;
        match command {
            Some("CAPA") => {
                stream.write(b"+OK Capability list follows\r\n").unwrap();
                stream.write(b"SASL CRAM-MD5 PLAIN\r\n").unwrap();
                stream.write(b"USER\r\n").unwrap();
                stream.write(b"PASS\r\n").unwrap();
                stream.write(b"AUTH\r\n").unwrap();
                stream.write(b"STAT\r\n").unwrap();
                stream.write(b"LIST\r\n").unwrap();
                stream.write(b"QUIT\r\n").unwrap();
                stream.write(b".\r\n").unwrap();
            }
            Some("AUTH") => {
                let method = args.next();
                match method {
                    Some("CRAM-MD5") => {
                        let timestamp = SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        let challenge = format!("<{}.{}@example.net>\r\n", timestamp, KEY);
                        let encoded_challenge = BASE64_STANDARD.encode(&challenge);
                        stream
                            .write(format!("+ {}\r\n", encoded_challenge).as_bytes())
                            .unwrap();
                        stream.flush().unwrap();

                        buffer.clear();
                        {
                            let mut reader = BufReader::new(&stream);
                            reader.read_line(&mut buffer).unwrap();
                        }
                        if !buffer.is_empty() {
                            let decoded_response = BASE64_STANDARD.decode(buffer.trim()).unwrap();
                            let response_str = String::from_utf8(decoded_response).unwrap();
                            
                            let mut parts = response_str.split_whitespace();
                            let username = parts.next().unwrap();
                            let password_hash = parts.next().unwrap();

                            match auth::validate_auth(username.to_string(), password_hash.to_string(), challenge) {
                                true => {
                                    session.user = Some(username.to_owned())
                                },
                                false => {
                                    stream.write(b"-ERR Invalid Credentials").unwrap();
                                    break;
                                },
                            }
                            // Respond to the client after validation
                            stream.write(b"+OK mailbox locked and ready\r\n").unwrap();
                        } else {
                            stream.write(b"-ERR Authentication failed\r\n").unwrap();
                        }
                    }
                    None => {
                        stream.write(b"+ Ok").unwrap();
                        stream.write(b".").unwrap();
                    }
                    _ => {
                        stream.write(b"-ERR unsupported auth method\r\n").unwrap();
                    }
                }
            }
            Some("USER") => {
                let user = args.next().unwrap();
                session.user = Some(user.to_string());
                stream.write(b"+OK User accepted\r\n").unwrap();
            }
            Some("PASS") => {
                let password = args.next().unwrap();
                session.password = Some(password.to_string());
                stream.write(b"+OK Password accepted\r\n").unwrap();
            }
            Some("STAT") => {
                match db::get_mailbox_size(session.user.as_ref().unwrap().to_string()) {
                    Err(_) => {
                        stream
                            .write(b"-ERR Could not retrieve mailbox size\r\n")
                            .unwrap();
                    }
                    Ok(size) => {
                        stream
                            .write(format!("+OK {} 0\r\n", size).as_bytes())
                            .unwrap();
                    }
                }
            }
            Some("LIST") => match db::get_mail(session.user.as_ref().unwrap().to_string()) {
                Err(e) => {
                    println!("Error while getting mailbox: {}", e);
                    stream
                        .write(b"-ERR Could not retrieve mailbox size\r\n")
                        .unwrap();
                }
                Ok(mails) => {
                    stream.write(b"+OK Mailbox listing follows\r\n").unwrap();
                    let mut i = 1;
                    for mail in mails {
                        stream
                            .write(format!("{} {}\r\n", i, mail.body.len()).as_bytes())
                            .unwrap();
                        i += 1;
                    }
                    stream.write(b".\r\n").unwrap();
                }
            },
            Some("RETR") => {
                let mail_id = args.next().unwrap().parse::<i32>().unwrap();
                match db::get_mail(session.user.as_ref().unwrap().to_string()) {
                    Err(_) => {
                        stream
                            .write(b"-ERR Could not retrieve mailbox size\r\n")
                            .unwrap();
                    }
                    Ok(mails) => {
                        let mail = &mails[mail_id as usize - 1];
                        stream.write(b"+OK Mail follows\r\n").unwrap();
                        stream.write(mail.body.as_bytes()).unwrap();
                        stream.write(b".\r\n").unwrap();
                    }
                }
            }
            Some("UIDL") => match db::get_mail(session.user.as_ref().unwrap().to_string()) {
                Err(_) => {
                    stream
                        .write(b"-ERR Could not retrieve mailbox size\r\n")
                        .unwrap();
                }
                Ok(mails) => {
                    stream.write(b"+OK Unique ID listing follows\r\n").unwrap();
                    let mut i = 1;
                    for mail in mails {
                        stream
                            .write(format!("{} {}\r\n", i, mail.uid).as_bytes())
                            .unwrap();
                        i += 1;
                    }
                    stream.write(b".\r\n").unwrap();
                }
            },
            Some("DELE") => {
                let mail_id = args.next().unwrap().parse::<i32>().unwrap();
                match delete_mail(mail_id) {
                    Ok(_) => {
                        stream.write(b"+OK msg deleted\r\n").unwrap();
                    }
                    Err(e) => {
                        stream.write(b"-ERR Could not delete message\r\n").unwrap();
                        println!("ERR: {}", e);
                    }
                }
            }
            Some("QUIT") => {
                stream.write(b"+OK POP3 server signing off\r\n").unwrap();
                break;
            }
            _ => {
                stream.write(b"-ERR Command not recognized\r\n").unwrap();
            }
        }
        buffer.clear();
    }
}
