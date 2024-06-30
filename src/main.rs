use std::net::{TcpListener, TcpStream};
use std::thread;

mod smtp;
mod pop3;
mod db;
mod models;
mod auth;

fn main() {
    let smtp_listener = TcpListener::bind("127.0.0.1:25").expect("Could not bind SMTP listener");
    let pop3_listener = TcpListener::bind("127.0.0.1:110").expect("Could not bind POP3 listener");
    
    db::init();
    println!("Database ready");

    println!("SMTP Server listening on port 25");
    println!("POP3 Server listening on port 110");

    // Spawn a new thread for the SMTP server
    thread::spawn(move || {
        for stream in smtp_listener.incoming() {
            match stream {
                Ok(stream) => {
                    handle_smtp_connection(stream);
                }
                Err(e) => {
                    println!("Error while creating SMTP handler: {}", e);
                }
            }
        }
    });

    // Main thread listens for POP3 connections
    for stream in pop3_listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_pop3_connection(stream);
            }
            Err(e) => {
                println!("Error while creating POP3 handler: {}", e);
            }
        }
    }
}

fn handle_smtp_connection(stream: TcpStream) {
    smtp::handle_connection(stream);
}

fn handle_pop3_connection(stream: TcpStream) {
    pop3::handle_connection(stream);
}