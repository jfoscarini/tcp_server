use std::{
    collections::{HashMap, HashSet},
    error::Error,
    io::ErrorKind,
    net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr},
    str::from_utf8,
};

use log::{error, trace, warn};
use mio::{event::Event, net::TcpListener, Events, Interest, Poll, Token};

use crate::client::Client;

pub enum ServerEvent {
    Connect(Token),
    Disconnect(Token),
    ReceiveUTF8(Token, String),
    ReceiveAdminUTF8(Token, String),
    ReceiveBytes(Token, Vec<u8>),
}

pub struct Server {
    tcp_listener: TcpListener,
    poll: Poll,
    current_token: Token,
    clients: HashMap<Token, Client>,
    admins: HashSet<Token>,
}

impl Server {
    pub fn step(&mut self, events: &mut Events) -> Result<Vec<ServerEvent>, Box<dyn Error>> {
        const SERVER: Token = Token(0);
        self.poll.poll(events, None)?;

        let mut response: Vec<ServerEvent> = Vec::new();

        for event in events.iter() {
            match event.token() {
                SERVER => {
                    if let Ok(Some(new_token)) = self.accept_incoming_connection() {
                        response.push(ServerEvent::Connect(new_token));
                    }
                }
                token => {
                    if let Some(new_event) = self.accept_client_event(&token, event) {
                        response.push(new_event);
                    }
                }
            }
        }

        Ok(response)
    }

    fn accept_client_event(&mut self, token: &Token, event: &Event) -> Option<ServerEvent> {
        if !event.is_readable() {
            return None;
        }

        if let Some(client) = self.clients.get_mut(token) {
            match client.read_from_buff() {
                Ok(Some(contents)) => {
                    if let Ok(str_buf) = from_utf8(&contents) {
                        let str_sanitized = str_buf
                            .trim()
                            .split_whitespace()
                            .collect::<Vec<_>>()
                            .join(" ");

                        match self.admins.contains(token) {
                            false if str_sanitized.eq(env!("GIT_HASH")) => {
                                trace!("({}) is now an admin.", token.0);
                                self.admins.insert(*token);

                                None
                            }
                            false => {
                                trace!("({}) received utf8 data: {}", token.0, str_sanitized);
                                Some(ServerEvent::ReceiveUTF8(*token, str_sanitized))
                            }
                            true => {
                                trace!("({}) received admin utf8 data: {}", token.0, str_sanitized);
                                Some(ServerEvent::ReceiveAdminUTF8(*token, str_sanitized))
                            }
                        }
                    } else {
                        trace!("({}) received (non UTF-8) data: {:?}", token.0, contents);
                        Some(ServerEvent::ReceiveBytes(*token, contents))
                    }
                }
                Ok(None) => Some(ServerEvent::Disconnect(*token)),
                Err(err) => {
                    error!("({}) error reading from buff: {:?}", token.0, err);
                    Some(ServerEvent::Disconnect(*token))
                }
            }
        } else {
            None
        }
    }

    fn accept_incoming_connection(&mut self) -> Result<Option<Token>, Box<dyn Error>> {
        match self.tcp_listener.accept() {
            Ok((mut stream, addr)) => {
                self.current_token.0 += 1;

                match self.poll.registry().register(
                    &mut stream,
                    self.current_token,
                    Interest::READABLE.add(Interest::WRITABLE),
                ) {
                    Ok(_) => {
                        self.clients.insert(
                            self.current_token,
                            Client::new(self.current_token, stream, addr),
                        );

                        trace!(
                            "({}) accepted connection from: {}",
                            self.current_token.0,
                            addr
                        );

                        Ok(Some(self.current_token))
                    }
                    Err(err) => {
                        stream.shutdown(Shutdown::Both).ok();

                        error!(
                            "({}) error registering new client: {:?}",
                            self.current_token.0, err
                        );

                        Ok(None)
                    }
                }
            }
            Err(err) if err.kind() == ErrorKind::WouldBlock => Ok(None),
            Err(err) => {
                error!("error accepting new client: {:?}", err);
                Err(Box::new(err))
            }
        }
    }

    pub fn get_client(&self, token: &Token) -> Option<&Client> {
        self.clients.get(&token)
    }

    pub fn get_client_mut(&mut self, token: &Token) -> Option<&mut Client> {
        self.clients.get_mut(&token)
    }

    pub fn disconnect(&mut self, token: &Token) {
        if let Some(mut client) = self.clients.remove(token) {
            trace!(
                "({}) closed connection from: {}",
                token.0,
                client.socket_address()
            );

            self.admins.remove(token);

            if self
                .poll
                .registry()
                .deregister(client.stream_mut())
                .is_err()
            {
                error!("({}) failed to de-register client", token.0);
            };
        } else {
            warn!("({}) could not disconnect client: not found", token.0);
        }
    }

    pub fn new(port: u16) -> Result<Server, Box<dyn Error>> {
        let local_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);

        let mut server = Server {
            tcp_listener: TcpListener::bind(local_addr)?,
            poll: Poll::new()?,
            current_token: Token(0),
            clients: HashMap::new(),
            admins: HashSet::new(),
        };

        server
            .poll
            .registry()
            .register(&mut server.tcp_listener, Token(0), Interest::READABLE)?;

        Ok(server)
    }
}
