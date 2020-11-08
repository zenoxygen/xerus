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
use crate::message::*;
use crate::peer::*;

use anyhow::{anyhow, Result};
use byteorder::{BigEndian, ReadBytesExt};

use std::io::{Cursor, Read, Write};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::time::Duration;

/// Client structure.
pub struct Client {
    // A peer
    peer: Peer,
    // Torrent peer id
    peer_id: Vec<u8>,
    // Torrent info hash
    info_hash: Vec<u8>,
    // Connection to peer
    conn: TcpStream,
}

impl Client {
    /// Build a new client.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - Urlencoded 20-byte string used as a unique ID for the client.
    /// * `info_hash` - 20-byte SHA-1 hash of the info key in the metainfo file.
    ///
    pub fn new(peer: Peer, peer_id: Vec<u8>, info_hash: Vec<u8>) -> Result<Client> {
        // Open connection with remote peer
        let peer_socket = SocketAddr::new(IpAddr::V4(peer.get_ip()), peer.get_port());
        let conn = match TcpStream::connect_timeout(&peer_socket, Duration::from_secs(15)) {
            Ok(conn) => conn,
            Err(_) => return Err(anyhow!("could not connect to peer")),
        };

        // Return new client
        let client = Client {
            peer,
            peer_id,
            info_hash,
            conn,
        };

        Ok(client)
    }

    /// Set connection timeout.
    ///
    /// # Arguments
    ///
    /// * `seconds` - The timeout in seconds.
    ///
    pub fn set_connection_timeout(&self, seconds: u64) -> Result<()> {
        // Set write timeout
        if self
            .conn
            .set_write_timeout(Some(Duration::from_secs(seconds)))
            .is_err()
        {
            return Err(anyhow!("could not set write timeout"));
        }

        // Set read timeout
        if self
            .conn
            .set_read_timeout(Some(Duration::from_secs(seconds)))
            .is_err()
        {
            return Err(anyhow!("could not set read timeout"));
        }

        Ok(())
    }

    /// Handshake with remote peer.
    pub fn handshake_with_peer(&mut self) -> Result<()> {
        // Set connection timeout
        self.set_connection_timeout(3)?;

        // Create handshake
        let peer_id = self.peer_id.clone();
        let info_hash = self.info_hash.clone();
        let handshake = Handshake::new(peer_id, info_hash)?;

        // Send handshake to remote peer
        let handshake_encoded: Vec<u8> = handshake.serialize()?;
        if self.conn.write(&handshake_encoded).is_err() {
            return Err(anyhow!("could not send handshake to peer"));
        }

        // Read handshake received from remote peer
        let handshake_len: u8 = self.read_handshake_len()?;
        let mut handshake_buf: Vec<u8> = vec![0; 48 + handshake_len as usize];
        if self.conn.read_exact(&mut handshake_buf).is_err() {
            return Err(anyhow!("could not parse handshake received from peer"));
        }

        // Check info hash received from remote peer
        let handshake_decoded: Handshake = deserialize_handshake(&handshake_buf, handshake_len)?;
        if handshake_decoded.get_info_hash() != self.info_hash {
            return Err(anyhow!("invalid handshake message received from peer"));
        }

        Ok(())
    }

    /// Read handshake length
    fn read_handshake_len(&mut self) -> Result<u8> {
        // Read 1 byte into buffer
        let mut buf = [0; 1];
        if self.conn.read_exact(&mut buf).is_err() {
            return Err(anyhow!(
                "could not read handshake length received from peer"
            ));
        }

        // Get handshake length
        let len = buf[0];
        if len == 0 {
            return Err(anyhow!("invalid handshake length received from peer"));
        }

        Ok(len)
    }

    /// Read message from remote peer.
    fn read_message(&mut self) -> Result<Message> {
        let message_len: u32 = self.read_message_len()?;
        let mut message_buf: Vec<u8> = vec![0; 4 + message_len as usize];
        if self.conn.read_exact(&mut message_buf).is_err() {
            return Err(anyhow!("could not parse message received from peer"));
        }

        let message: Message = deserialize_message(&message_buf, message_len)?;

        Ok(message)
    }

    /// Read message length.
    fn read_message_len(&mut self) -> Result<u32> {
        // Read bytes into buffer
        let mut buf = [0; 4];
        if self.conn.read_exact(&mut buf).is_err() {
            return Err(anyhow!("could not read message length received from peer"));
        }

        // Get message length
        let mut buf_cursor = Cursor::new(buf);
        let len = buf_cursor.read_u32::<BigEndian>()?;

        // If message length is 0, it's a keep-alive
        if len == 0 {
            println!("Received keep-alive message");
        }

        Ok(len)
    }

    /// Read BITFIELD message from remote peer.
    pub fn read_bitfield(&mut self) -> Result<Message> {
        println!("Read bitfield from {:?}:{:?}", self.peer.ip, self.peer.port);
        let message: Message = self.read_message()?;
        if message.get_id() != MESSAGE_BITFIELD {
            return Err(anyhow!("could not read MESSAGE_BITFIELD from peer"));
        }

        Ok(message)
    }

    /// Send CHOKE message to remote peer.
    pub fn send_choke(&mut self) -> Result<()> {
        let message: Message = Message::new(MESSAGE_CHOKE)?;
        let message_encoded = message.serialize()?;
        println!(
            "Send MESSAGE_CHOKE to {:?}:{:?}",
            self.peer.ip, self.peer.port
        );
        if self.conn.write(&message_encoded).is_err() {
            return Err(anyhow!("could not send MESSAGE_CHOKE to peer"));
        }

        Ok(())
    }

    /// Send UNCHOKE message to remote peer.
    pub fn send_unchoke(&mut self) -> Result<()> {
        let message: Message = Message::new(MESSAGE_UNCHOKE)?;
        let message_encoded = message.serialize()?;
        println!(
            "Send MESSAGE_UNCHOKE to {:?}:{:?}",
            self.peer.ip, self.peer.port
        );
        if self.conn.write(&message_encoded).is_err() {
            return Err(anyhow!("could not send MESSAGE_UNCHOKE to peer"));
        }

        Ok(())
    }

    /// Send INTERESTED message to remote peer.
    pub fn send_interested(&mut self) -> Result<()> {
        let message: Message = Message::new(MESSAGE_INTERESTED)?;
        let message_encoded = message.serialize()?;
        println!(
            "Send MESSAGE_INTERESTED to {:?}:{:?}",
            self.peer.ip, self.peer.port
        );
        if self.conn.write(&message_encoded).is_err() {
            return Err(anyhow!("could not send MESSAGE_INTERESTED to peer"));
        }

        Ok(())
    }

    /// Send NOT INTERESTED message to remote peer.
    pub fn send_not_interested(&mut self) -> Result<()> {
        let message: Message = Message::new(MESSAGE_NOT_INTERESTED)?;
        let message_encoded = message.serialize()?;
        println!(
            "Send MESSAGE_NOT_INTERESTED to {:?}:{:?}",
            self.peer.ip, self.peer.port
        );
        if self.conn.write(&message_encoded).is_err() {
            return Err(anyhow!("could not send MESSAGE_NOT_INTERESTED to peer"));
        }

        Ok(())
    }

    /// Send HAVE message to remote peer.
    pub fn send_have(&mut self) -> Result<()> {
        let message: Message = Message::new(MESSAGE_HAVE)?;
        let message_encoded = message.serialize()?;
        println!(
            "Send MESSAGE_HAVE to {:?}:{:?}",
            self.peer.ip, self.peer.port
        );
        if self.conn.write(&message_encoded).is_err() {
            return Err(anyhow!("could not send MESSAGE_HAVE to peer"));
        }

        Ok(())
    }

    /// Send BITFIELD message to remote peer.
    pub fn send_bitfield(&mut self) -> Result<()> {
        let message: Message = Message::new(MESSAGE_BITFIELD)?;
        let message_encoded = message.serialize()?;
        println!(
            "Send MESSAGE_BITFIELD Tto {:?}:{:?}",
            self.peer.ip, self.peer.port
        );
        if self.conn.write(&message_encoded).is_err() {
            return Err(anyhow!("could not send MESSAGE_BITFIELD to peer"));
        }

        Ok(())
    }

    /// Send REQUEST message to remote peer.
    pub fn send_request(&mut self) -> Result<()> {
        let message: Message = Message::new(MESSAGE_REQUEST)?;
        let message_encoded = message.serialize()?;
        println!(
            "Send MESSAGE_REQUEST to {:?}:{:?}",
            self.peer.ip, self.peer.port
        );
        if self.conn.write(&message_encoded).is_err() {
            return Err(anyhow!("could not send MESSAGE_REQUEST to peer"));
        }

        Ok(())
    }

    /// Send PIECE message to remote peer.
    pub fn send_piece(&mut self) -> Result<()> {
        let message: Message = Message::new(MESSAGE_PIECE)?;
        let message_encoded = message.serialize()?;
        println!(
            "Send MESSAGE_PIECE to {:?}:{:?}",
            self.peer.ip, self.peer.port
        );
        if self.conn.write(&message_encoded).is_err() {
            return Err(anyhow!("could not send MESSAGE_PIECE to peer"));
        }

        Ok(())
    }

    /// Send CANCEL message to remote peer.
    pub fn send_cancel(&mut self) -> Result<()> {
        let message: Message = Message::new(MESSAGE_CANCEL)?;
        let message_encoded = message.serialize()?;
        println!(
            "Send MESSAGE_CANCEL to {:?}:{:?}",
            self.peer.ip, self.peer.port
        );
        if self.conn.write(&message_encoded).is_err() {
            return Err(anyhow!("could not send MESSAGE_CANCEL to peer"));
        }

        Ok(())
    }

    /// Send PORT message to remote peer.
    pub fn send_port(&mut self) -> Result<()> {
        let message: Message = Message::new(MESSAGE_PORT)?;
        let message_encoded = message.serialize()?;
        println!(
            "Send MESSAGE_PORT to {:?}:{:?}",
            self.peer.ip, self.peer.port
        );
        if self.conn.write(&message_encoded).is_err() {
            return Err(anyhow!("could not send MESSAGE_PORT to peer"));
        }

        Ok(())
    }
}
