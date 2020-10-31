// Copyright (c) 2020 zenoxygen
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

extern crate anyhow;
extern crate url;

use crate::handshake::*;
use crate::peer::*;

use anyhow::{anyhow, Result};

use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::time::Duration;

/// Client structure.
pub struct Client {
    peer_id: Vec<u8>,
    info_hash: Vec<u8>,
    pub conn: TcpStream,
}

impl Client {
    /// Build a new client.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - Urlencoded 20-byte string used as a unique ID for the client.
    /// * `info_hash` - 20-byte SHA-1 hash of the info key in the metainfo file.
    ///
    pub fn new(peer: &Peer, peer_id: Vec<u8>, info_hash: Vec<u8>) -> Result<Client> {
        // Open connection with remote peer
        let peer_socket = SocketAddr::new(IpAddr::V4(peer.ip), peer.port);
        let conn = match TcpStream::connect_timeout(&peer_socket, Duration::from_secs(15)) {
            Ok(conn) => conn,
            Err(_) => return Err(anyhow!("could not connect to peer")),
        };

        // Set write timeout
        if conn
            .set_write_timeout(Some(Duration::from_secs(3)))
            .is_err()
        {
            return Err(anyhow!("could not set write timeout"));
        }

        // Set read timeout
        if conn.set_read_timeout(Some(Duration::from_secs(3))).is_err() {
            return Err(anyhow!("could not set read timeout"));
        }

        // Return new client
        let client = Client {
            peer_id,
            info_hash,
            conn,
        };

        Ok(client)
    }

    /// Handshake with remote peer.
    pub fn handshake_with_peer(&mut self) -> Result<()> {
        let handshake = Handshake::new(self.peer_id.clone(), self.info_hash.clone())?;
        let handshake_encoded: Vec<u8> = handshake.serialize()?;

        // Send handshake message to peer
        if self.conn.write(&handshake_encoded).is_err() {
            return Err(anyhow!("could not send handshake to peer"));
        }

        // Read buf size
        let mut buf = [0; 1];
        if self.conn.read_exact(&mut buf).is_err() {
            return Err(anyhow!("could not parse handshake received from peer"));
        }

        // Check buf size
        let buf_size = buf[0];
        if buf_size == 0 {
            return Err(anyhow!("invalid handshake message received from peer"));
        }

        // Read handshake message received from peer
        let mut buf: Vec<u8> = vec![0; 48 + buf_size as usize];
        if self.conn.read_exact(&mut buf).is_err() {
            return Err(anyhow!("could not parse handshake received from peer"));
        }

        // Deserialize handshake message
        let handshake_decoded: Handshake = deserialize(&buf, buf_size)?;

        // Check info hash received
        if handshake_decoded.info_hash != self.info_hash {
            return Err(anyhow!("invalid handshake message received from peer"));
        }

        Ok(())
    }
}
