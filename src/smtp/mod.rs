use std::net::TcpStream;
mod handler;

pub fn handle_connection(stream: TcpStream) {
   handler::handle(stream);
}

