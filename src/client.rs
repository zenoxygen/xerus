extern crate anyhow;
extern crate url;

use crate::peers::*;

use anyhow::{anyhow, Result};

use std::net::{IpAddr, SocketAddr, TcpStream};
use std::time::Duration;

/// Client structure.
pub struct Client {
    pub conn: TcpStream,
}

impl Client {
    pub fn new(peer: &Peer, peer_id: Vec<u8>, info_hash: Vec<u8>) -> Result<Client> {
        println!("Connecting to peer {:?}:{:?}", &peer.ip, &peer.port);
        let peer_socket = SocketAddr::new(IpAddr::V4(peer.ip), peer.port);
        let peer_conn = TcpStream::connect_timeout(&peer_socket, Duration::from_secs(15))?;
        let client = Client { conn: peer_conn };
        Ok(client)
    }
}
