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

use crate::peer::*;
use crate::piece::*;
use crate::worker::*;

use anyhow::{anyhow, Result};
use boring::sha::Sha1;
use crossbeam_channel::{unbounded, Receiver, Sender};
use indicatif::{ProgressBar, ProgressStyle};
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
use std::thread;
use std::time::Duration;

const PORT: u16 = 6881;
const SHA1_HASH_SIZE: usize = 20;

/// Torrent structure.
#[derive(Default, Clone)]
pub struct Torrent {
    // URL of the tracker
    announce: String,
    // 20-byte SHA-1 hash calculated over the content of the bencoded info dictionary
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
    fn hash(&self) -> Result<Vec<u8>> {
        // Serialize bencoded informations
        let buf: Vec<u8> = ser::to_bytes::<BencodeInfo>(self)?;

        // Hash bencoded informations
        let mut hasher = Sha1::new();
        hasher.update(&buf);

        // Read hash digest
        let hash = hasher.finish().to_vec();

        Ok(hash)
    }

    /// Split bencoded pieces into vectors of SHA-1 hashes.
    fn split_pieces_hashes(&self) -> Result<Vec<Vec<u8>>> {
        let pieces = self.pieces.to_owned();
        let nb_pieces = pieces.len();

        // Check torrent pieces
        if nb_pieces % SHA1_HASH_SIZE != 0 {
            return Err(anyhow!("torrent is invalid"));
        }
        let nb_hashes = nb_pieces / SHA1_HASH_SIZE;
        let mut hashes: Vec<Vec<u8>> = vec![vec![0; 20]; nb_hashes];

        // Split pieces
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
        let mut file = match File::open(filepath) {
            Ok(file) => file,
            Err(_) => return Err(anyhow!("could not open torrent")),
        };

        // Read torrent content in a buffer
        let mut buf = vec![];
        if file.read_to_end(&mut buf).is_err() {
            return Err(anyhow!("could not read torrent"));
        }
        // Deserialize bencoded data from torrent
        let bencode = match de::from_bytes::<BencodeTorrent>(&buf) {
            Ok(bencode) => bencode,
            Err(_) => return Err(anyhow!("could not decode torrent")),
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

    /// Request peers from tracker.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - Urlencoded 20-byte string used as a unique ID for the client.
    /// * `port` - Port number that the client is listening on.
    ///
    fn request_peers(&self, peer_id: Vec<u8>, port: u16) -> Result<Vec<Peer>> {
        // Build tracker URL
        let tracker_url = match self.build_tracker_url(peer_id, port) {
            Ok(url) => url,
            Err(_) => return Err(anyhow!("could not build tracker url")),
        };

        // Build blocking HTTP client
        let client = match reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
        {
            Ok(client) => client,
            Err(_) => return Err(anyhow!("could not connect to tracker")),
        };

        // Send GET request to the tracker
        let response = match client.get(&tracker_url).send() {
            Ok(response) => match response.bytes() {
                Ok(bytes) => bytes,
                Err(_) => return Err(anyhow!("could not read response from tracker")),
            },
            Err(_) => return Err(anyhow!("could not send request to tracker")),
        };

        // Deserialize bencoded tracker response
        let tracker_bencode = match de::from_bytes::<BencodeTracker>(&response) {
            Ok(bencode) => bencode,
            Err(_) => return Err(anyhow!("could not decode tracker response")),
        };

        // Build peers from tracker response
        let peers: Vec<Peer> = match self.build_peers(tracker_bencode.peers.to_vec()) {
            Ok(peers) => peers,
            Err(_) => return Err(anyhow!("could not build peers")),
        };

        Ok(peers)
    }

    /// Build tracker URL.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - Urlencoded 20-byte string used as a unique ID for the client.
    /// * `port` - Port number that the client is listening on.
    ///
    fn build_tracker_url(&self, peer_id: Vec<u8>, port: u16) -> Result<String> {
        // Parse tracker URL from torrent
        let mut base_url = match Url::parse(&self.announce) {
            Ok(url) => url,
            Err(_) => return Err(anyhow!("could not parse tracker url")),
        };

        // Add parameters to the tracker URL
        base_url
            // Add info hash
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
            // Add peer id
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
            // Add port
            .append_pair("port", &port.to_string())
            // Add uploaded
            .append_pair("uploaded", "0")
            // Add downloaded
            .append_pair("downloaded", "0")
            // Add compact
            .append_pair("compact", "1")
            // Add left
            .append_pair("left", &self.length.to_string());

        Ok(base_url.to_string())
    }

    /// Download torrent.
    pub fn download(&self) -> Result<Vec<u8>> {
        println!(
            "Downloading {:?} ({:?} pieces)",
            self.name,
            self.pieces_hashes.len(),
        );

        // Create work pieces channel
        let work_chan: (Sender<PieceWork>, Receiver<PieceWork>) = unbounded();

        // Create result pieces channel
        let result_chan: (Sender<PieceResult>, Receiver<PieceResult>) = unbounded();

        // Create and send pieces to work channel
        for index in 0..self.pieces_hashes.len() {
            // Create piece
            let piece_index = index as u32;
            let piece_hash = self.pieces_hashes[index].clone();
            let piece_length = self.get_piece_length(piece_index)?;
            let piece_work = PieceWork::new(piece_index, piece_hash, piece_length);

            // Send piece to work channel
            if work_chan.0.send(piece_work).is_err() {
                return Err(anyhow!("Error: could not send piece to channel"));
            }
        }

        // Init workers
        let peers = self.peers.to_owned();
        for peer in peers {
            let peer_copy = peer.clone();
            let peer_id_copy = self.peer_id.clone();
            let info_hash_copy = self.info_hash.clone();
            let work_chan_copy = work_chan.clone();
            let result_chan_copy = result_chan.clone();

            // Create new worker
            let worker = Worker::new(
                peer_copy,
                peer_id_copy,
                info_hash_copy,
                work_chan_copy,
                result_chan_copy,
            )?;

            // Start worker in a new thread
            thread::spawn(move || {
                worker.start_download();
            });
        }

        // Create progress bar
        let pb = ProgressBar::new(self.length as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} {bytes}/{total_bytes} [{bar:40.cyan/blue}] {percent}%")
                .unwrap()
                .progress_chars("#>-"),
        );

        // Build torrent
        let mut data: Vec<u8> = vec![0; self.length as usize];
        let mut nb_pieces_downloaded = 0;
        while nb_pieces_downloaded < self.pieces_hashes.len() {
            // Receive a piece from result channel
            let piece_result: PieceResult = match result_chan.1.recv() {
                Ok(piece_result) => piece_result,
                Err(_) => return Err(anyhow!("Error: could not receive piece from channel")),
            };

            // Copy piece data
            let begin: u32 = piece_result.index * self.piece_length;
            for i in 0..piece_result.length as usize {
                data[begin as usize + i] = piece_result.data[i];
            }

            // Update progress bar
            pb.inc(piece_result.length as u64);

            // Update number of pieces downloaded
            nb_pieces_downloaded += 1;
        }

        Ok(data)
    }

    /// Get piece length.
    ///
    /// # Arguments
    ///
    /// * `index` - The piece index.
    ///
    fn get_piece_length(&self, index: u32) -> Result<u32> {
        let begin: u32 = index * self.piece_length;
        let mut end: u32 = begin + self.piece_length;

        // Prevent unbounded values
        if end > self.length {
            end = self.length;
        }

        Ok(end - begin)
    }
}
