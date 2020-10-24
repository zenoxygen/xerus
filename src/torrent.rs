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
extern crate crypto;
extern crate hex;
extern crate serde;
extern crate serde_bencode;
extern crate url;

use crate::client::*;
use crate::peers::*;

use anyhow::{anyhow, Result};
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_bencode::{de, ser};
use serde_bytes::ByteBuf;
use std::str;
use url::Url;

use std::borrow::Cow;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;

const PORT: u16 = 6881;
const SHA1_HASH_SIZE: usize = 20;

/// Torrent structure.
#[derive(Default)]
pub struct Torrent {
    // URL of the tracker
    announce: String,
    // SHA-1 hash calculated over the content of the bencoded info dictionary
    info_hash: Vec<u8>,
    // SHA-1 hashes of each pieces
    pieces_hashes: Vec<Vec<u8>>,
    // Size of each piece in bytes
    piece_length: u32,
    // Size of the file in bytes
    length: u32,
    // Suggested filename where to save the file
    name: String,
    // Urlencoded 20-byte string used as unique client ID
    peer_id: Vec<u8>,
    // Peers
    peers: Vec<Peer>,
}

/// BencodeInfo structure.
#[derive(Deserialize, Serialize)]
struct BencodeInfo {
    // Concatenation of all pieces 20-byte SHA-1 hashes
    #[serde(rename = "pieces")]
    pieces: ByteBuf,
    // Size of each piece in bytes
    #[serde(rename = "piece length")]
    piece_length: u32,
    // Size of the file in bytes
    #[serde(rename = "length")]
    length: u32,
    // Suggested filename where to save the file
    #[serde(rename = "name")]
    name: String,
}

/// BencodeTorrent structure.
#[derive(Deserialize, Serialize)]
struct BencodeTorrent {
    #[serde(default)]
    // URL of the tracker
    announce: String,
    // Informations about file
    info: BencodeInfo,
}

/// BencodeTracker structure.
#[derive(Debug, Deserialize, Serialize)]
struct BencodeTracker {
    // Interval time to refresh the list of peers in seconds
    interval: u32,
    // Peers IP addresses
    peers: ByteBuf,
}

impl BencodeInfo {
    /// Hash bencoded informations to uniquely identify a file.
    pub fn hash(&self) -> Result<Vec<u8>> {
        // Serialize bencoded informations
        let buf: Vec<u8> = ser::to_bytes::<BencodeInfo>(self)?;
        // Hash bencoded informations
        let mut hasher = Sha1::new();
        hasher.input(&buf);
        let hex = hasher.result_str();
        let decoded: Vec<u8> = hex::decode(hex)?;

        Ok(decoded)
    }

    /// Split bencoded pieces into vectors of SHA-1 hashes.
    pub fn split_pieces_hashes(&self) -> Result<Vec<Vec<u8>>> {
        let pieces = self.pieces.to_owned();
        let nb_pieces = pieces.len();
        // Check torrent pieces
        if nb_pieces % SHA1_HASH_SIZE != 0 {
            return Err(anyhow!("torrent is invalid"));
        }
        let nb_hashes = nb_pieces / SHA1_HASH_SIZE;
        let mut hashes: Vec<Vec<u8>> = vec![vec![0; 20]; nb_hashes];
        for i in 0..nb_hashes {
            hashes[i] = pieces[i * SHA1_HASH_SIZE..(i + 1) * SHA1_HASH_SIZE].to_vec();
        }

        Ok(hashes)
    }
}

impl Torrent {
    /// Build a new torrent.
    pub fn new() -> Self {
        Default::default()
    }

    /// Open torrent.
    ///
    /// # Arguments
    ///
    /// * `filepath` - Path to the torrent.
    ///
    pub fn open(&mut self, filepath: PathBuf) -> Result<()> {
        // Open torrent
        let file = File::open(filepath);
        let mut file = match file {
            Ok(file) => file,
            Err(_) => return Err(anyhow!("could not open torrent")),
        };

        // Read torrent content in a buffer
        let mut buf = Vec::new();
        let bencode = match file.read_to_end(&mut buf) {
            // Deserialize bencoded data from torrent
            Ok(_) => match de::from_bytes::<BencodeTorrent>(&buf) {
                Ok(bencode) => bencode,
                Err(_) => return Err(anyhow!("could not decode torrent")),
            },
            Err(_) => return Err(anyhow!("could not read torrent")),
        };

        // Generate a random 20-byte peer id
        let mut peer_id: Vec<u8> = vec![0; 20];
        let mut rng = rand::thread_rng();
        for x in peer_id.iter_mut() {
            *x = rng.gen();
        }

        // Add torrent informations
        self.announce = bencode.announce.to_owned();
        self.info_hash = bencode.info.hash()?;
        self.pieces_hashes = bencode.info.split_pieces_hashes()?;
        self.piece_length = bencode.info.piece_length;
        self.length = bencode.info.length;
        self.name = bencode.info.name.to_owned();
        self.peer_id = peer_id.clone();
        self.peers = self.request_peers(peer_id, PORT)?;

        Ok(())
    }

    /// Download torrent.
    ///
    /// # Arguments
    ///
    /// * `filepath` - Path where to save the file.
    ///
    pub fn download(&mut self, filepath: PathBuf) -> Result<()> {
        println!("Downloading torrent...");
        for peer in self.peers.iter_mut() {
            let peer_id = self.peer_id.clone();
            let info_hash = self.info_hash.clone();
            if let Ok(client) = Client::new(&peer, peer_id, info_hash) {
                println!("Connected to peer {:?}:{:?}", peer.ip, peer.port);
            } else {
                eprintln!("Unable to connect to peer");
            }
        }

        Ok(())
    }

    /// Request peers from tracker.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - Urlencoded 20-byte string used as a unique ID for the client.
    /// * `port` - Port number that the client is listening on.
    ///
    pub fn request_peers(&self, peer_id: Vec<u8>, port: u16) -> Result<Vec<Peer>> {
        // Build tracker URL
        let tracker_url = match self.build_tracker_url(peer_id, port) {
            Ok(url) => url,
            Err(_) => return Err(anyhow!("could not build tracker url")),
        };

        // Build blocking HTTP client
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()?;

        // Send GET request to the tracker
        println!("Send request to torrent tracker...");
        let response = client.get(&tracker_url).send().unwrap().bytes()?;

        // Deserialize bencoded tracker response
        let bencode_tracker = match de::from_bytes::<BencodeTracker>(&response) {
            Ok(bencode) => bencode,
            Err(_) => return Err(anyhow!("could not decode tracker response")),
        };

        // Build peers from tracker response
        let peers: Vec<Peer> = build_peers(bencode_tracker.peers.to_vec())?;

        Ok(peers)
    }

    /// Build tracker URL.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - Urlencoded 20-byte string used as a unique ID for the client.
    /// * `port` - Port number that the client is listening on.
    ///
    pub fn build_tracker_url(&self, peer_id: Vec<u8>, port: u16) -> Result<String> {
        // Parse tracker URL from torrent
        let mut base_url = Url::parse(&self.announce)?;

        // Add parameters to the tracker URL
        base_url
            .query_pairs_mut()
            .encoding_override(Some(&|input| {
                if input != "!" {
                    Cow::Borrowed(input.as_bytes())
                } else {
                    Cow::Owned(self.info_hash.clone())
                }
            }))
            .append_pair("info_hash", "!");
        base_url
            .query_pairs_mut()
            .encoding_override(Some(&|input| {
                if input != "!" {
                    Cow::Borrowed(input.as_bytes())
                } else {
                    Cow::Owned(peer_id.clone())
                }
            }))
            .append_pair("peer_id", "!");
        base_url
            .query_pairs_mut()
            .append_pair("port", &port.to_string())
            .append_pair("uploaded", "0")
            .append_pair("downloaded", "0")
            .append_pair("compact", "1")
            .append_pair("left", &self.length.to_string());

        Ok(base_url.to_string())
    }
}
