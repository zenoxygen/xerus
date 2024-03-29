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

use crate::torrent::*;

use anyhow::{anyhow, Result};
use byteorder::{BigEndian, ReadBytesExt};

use std::io::Cursor;
use std::net::Ipv4Addr;

const PEER_SIZE: usize = 6;

type PeerId = u32;

/// Peer structure.
#[derive(Clone)]
pub struct Peer {
    pub id: PeerId,
    pub ip: Ipv4Addr,
    pub port: u16,
}

impl Peer {
    /// Build a new peer.
    pub fn new() -> Peer {
        Peer {
            id: 0,
            ip: Ipv4Addr::new(1, 1, 1, 1),
            port: 0,
        }
    }
}

impl Torrent {
    /// Build peers.
    ///
    /// # Arguments
    ///
    /// * `tracker_peers` - A string consisting of multiples of 6 bytes.
    /// First 4 bytes are the IP address and last 2 bytes are the port number.
    /// All in network (big endian) notation.
    ///
    pub fn build_peers(&self, tracker_peers: Vec<u8>) -> Result<Vec<Peer>> {
        // Check tracker peers are valid
        if tracker_peers.len() % PEER_SIZE != 0 {
            return Err(anyhow!("received invalid peers from tracker"));
        }

        // Get number of peers
        let nb_peers = tracker_peers.len() / PEER_SIZE;

        // Build peers
        let mut peers: Vec<Peer> = vec![Peer::new(); nb_peers];

        for (i, peer) in peers.iter_mut().enumerate().take(nb_peers) {
            // Create peer ID
            peer.id = i as u32;

            let offset = i * PEER_SIZE;

            // Read peer IP address
            peer.ip = Ipv4Addr::new(
                tracker_peers[offset],
                tracker_peers[offset + 1],
                tracker_peers[offset + 2],
                tracker_peers[offset + 3],
            );

            // Read peer port
            let port_bytes = &tracker_peers[offset + 4..offset + 6];
            let mut port_cursor = Cursor::new(port_bytes);
            peer.port = port_cursor.read_u16::<BigEndian>()?;
        }

        Ok(peers)
    }
}
