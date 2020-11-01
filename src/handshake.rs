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

use anyhow::Result;
use serde::{Deserialize, Serialize};

const PROTOCOL_ID: &str = "BitTorrent protocol";

/// Handshake structure.
#[derive(Deserialize, Serialize)]
pub struct Handshake {
    pub pstrlen: u8,
    // String identifier of the protocol
    pub pstr: Vec<u8>,
    // 8 reserved bytes, all set to 0
    pub reserved: Vec<u8>,
    // 20-byte SHA-1 hash of the info key in the metainfo file
    pub info_hash: Vec<u8>,
    // 20-byte string used as a unique ID for the client
    pub peer_id: Vec<u8>,
}

impl Handshake {
    /// Build a new handshake message.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - Urlencoded 20-byte string used as a unique ID for the client.
    /// * `info_hash` - 20-byte SHA-1 hash of the info key in the metainfo file.
    ///
    pub fn new(peer_id: Vec<u8>, info_hash: Vec<u8>) -> Result<Handshake> {
        // Get pstr
        let pstr = String::from(PROTOCOL_ID).into_bytes();
        // Get pstrlen
        let pstrlen = pstr.len() as u8;
        // Get reserved
        let reserved: Vec<u8> = vec![0; 8];

        // Build handshake
        let handshake = Handshake {
            pstrlen,
            pstr,
            reserved,
            info_hash,
            peer_id,
        };

        Ok(handshake)
    }

    /// Serialize an handshake message.
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut serialized: Vec<u8> = vec![];

        // Add pstrlen
        serialized.push(self.pstrlen);

        // Add pstr
        let mut pstr: Vec<u8> = self.pstr.clone();
        serialized.append(&mut pstr);

        // Add reserved
        let mut reserved: Vec<u8> = self.reserved.clone();
        serialized.append(&mut reserved);

        // Add info hash
        let mut info_hash: Vec<u8> = self.info_hash.clone();
        serialized.append(&mut info_hash);

        // Add peer id
        let mut peer_id: Vec<u8> = self.peer_id.clone();
        serialized.append(&mut peer_id);

        Ok(serialized)
    }
}

/// Deserialize an handshake message.
///
/// # Arguments
///
/// * `buf_handshake` - Bytes containing an handshake message.
/// * `pstrlen` - The length of the protocol identifier.
///
pub fn deserialize_handshake(buf_handshake: &Vec<u8>, pstrlen: u8) -> Result<Handshake> {
    let mut pstr = Vec::new();
    let mut reserved = vec![0; 8];
    let mut info_hash = Vec::new();
    let mut peer_id = Vec::new();

    for (i, x) in buf_handshake.iter().enumerate() {
        // Get pstr
        if i < pstrlen as usize {
            pstr.push(x.to_owned());
        }
        // Get reserved
        if i >= (pstrlen as usize) && i < (pstrlen as usize + 8) {
            reserved.push(x.to_owned());
        }
        // Get info hash
        if i >= (pstrlen as usize + 8) && i < (pstrlen as usize + 8 + 20) {
            info_hash.push(x.to_owned());
        }
        // Get peer id
        if i >= (pstrlen as usize + 8 + 20) {
            peer_id.push(x.to_owned());
        }
    }

    // Build handshake
    let handshake = Handshake {
        pstrlen,
        pstr,
        reserved,
        info_hash,
        peer_id,
    };

    Ok(handshake)
}
