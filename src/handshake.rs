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

const PROTOCOL_ID: &str = "BitTorrent protocol";

/// Handshake structure.
pub struct Handshake {
    pstrlen: usize,
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
    pub fn new(peer_id: Vec<u8>, info_hash: Vec<u8>) -> Self {
        // Get pstr
        let pstr = String::from(PROTOCOL_ID).into_bytes();
        // Get pstrlen
        let pstrlen = pstr.len();
        // Get reserved
        let reserved: Vec<u8> = vec![0; 8];

        Handshake {
            pstrlen,
            pstr,
            reserved,
            info_hash,
            peer_id,
        }
    }

    // Get handshake info hash.
    pub fn get_info_hash(self) -> Vec<u8> {
        self.info_hash
    }

    /// Serialize an handshake message.
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut serialized: Vec<u8> = vec![];

        // Add pstrlen
        serialized.push(self.pstrlen as u8);

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

/// Deserialize a handshake message.
///
/// # Arguments
///
/// * `buf` - Bytes containing the handshake message.
/// * `pstrlen` - Length of protocol identifier.
///
pub fn deserialize_handshake(buf: &Vec<u8>, pstrlen: usize) -> Result<Handshake> {
    // Get pstr
    let pstr = buf[0..pstrlen].to_vec();
    // Get reserved
    let reserved = buf[pstrlen..(pstrlen + 8)].to_vec();
    // Get info hash
    let info_hash = buf[(pstrlen + 8)..(pstrlen + 8 + 20)].to_vec();
    // Get peer id
    let peer_id = buf[(pstrlen + 8 + 20)..].to_vec();

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
