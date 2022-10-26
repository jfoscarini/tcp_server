use std::{
    error::Error,
    io::{ErrorKind, Read, Write},
    net::SocketAddr,
};

use log::{error, trace};
use mio::{net::TcpStream, Token};

pub struct Client {
    token: Token,
    stream: TcpStream,
    addr: SocketAddr,
}

impl Client {
    pub fn stream_mut(&mut self) -> &mut TcpStream {
        &mut self.stream
    }

    pub fn socket_address(&self) -> &SocketAddr {
        &self.addr
    }

    pub fn token(&self) -> &Token {
        &self.token
    }

    pub fn write_str(&mut self, msg: &str) {
        match self.stream.write(msg.as_bytes()) {
            Ok(_) => {
                trace!(
                    "({}) sent utf8 data: {}",
                    self.token.0,
                    msg.replace("\n", "\\n").replace("\r", "\\r")
                );
            }
            Err(err) => {
                error!("({}) could not write: {:?}", self.token.0, err);
            }
        }
    }

    pub fn write_bytes(&mut self, msg: &[u8]) {
        match self.stream.write(msg) {
            Ok(_) => {
                trace!("({}) sent raw data: {:?}", self.token.0, msg);
            }
            Err(err) => {
                error!("({}) could not write: {:?}", self.token.0, err);
            }
        }
    }

    pub fn read_from_buff(&mut self) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
        let mut received_data = vec![0; 4096];
        let mut bytes_read = 0;

        loop {
            match self.stream.read(&mut received_data[bytes_read..]) {
                Ok(0) => return Ok(None),
                Ok(n) => {
                    bytes_read += n;
                    if bytes_read == received_data.len() {
                        received_data.resize(received_data.len() + 1024, 0);
                    }
                }
                Err(ref err) if err.kind() == ErrorKind::WouldBlock => break,
                Err(ref err) if err.kind() == ErrorKind::Interrupted => continue,
                Err(err) => {
                    return Err(Box::new(err));
                }
            }
        }

        Ok(Some(received_data[..bytes_read].to_vec()))
    }

    pub fn new(token: Token, stream: TcpStream, addr: SocketAddr) -> Client {
        Client {
            token,
            stream,
            addr,
        }
    }
}
