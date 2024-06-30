use std::net::TcpStream;
mod handler;
mod session;

pub fn handle_connection(stream: TcpStream) {
    handler::handle(stream);
}