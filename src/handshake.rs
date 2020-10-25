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

use anyhow::{anyhow, Result};

use serde::{Deserialize, Serialize};

const PROTOCOL_ID: &str = "BitTorrent protocol";

/// Handshake structure.
#[derive(Serialize, Deserialize)]
pub struct Handshake {
    // The length of the protocol identifier
    pstrlen: u8,
    // String identifier of the protocol
    pstr: Vec<u8>,
    // 8 reserved bytes, all set to 0
    reserved: Vec<u8>,
    // 20-byte SHA-1 hash of the info key in the metainfo file
    info_hash: Vec<u8>,
    // 20-byte string used as a unique ID for the client
    peer_id: Vec<u8>,
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
        let pstr = String::from(PROTOCOL_ID).into_bytes();
        let pstrlen = pstr.len() as u8;
        let reserved: Vec<u8> = vec![0; 8];
        let handshake = Handshake {
            pstrlen,
            pstr,
            reserved,
            info_hash,
            peer_id,
        };

        Ok(handshake)
    }

    /// Serialize the handshake message.
    ///
    /// The handshake message sent to peers is the concatenation of ‘pstrlen’, ‘pstr’, ‘reserved’, ‘info_hash’, and ‘peer_id’ into one long byte string.
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let encoded: Vec<u8> = match bincode::serialize(self) {
            Ok(encoded) => encoded,
            Err(_) => return Err(anyhow!("could not serialize handshake message")),
        };

        Ok(encoded)
    }
}
