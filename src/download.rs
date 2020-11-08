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

use crate::bitfield::*;
use crate::client::*;
use crate::peer::*;
use crate::piece::*;
use crate::torrent::*;

use anyhow::{anyhow, Result};
use crossbeam_channel::{unbounded, Receiver, Sender};

use std::path::PathBuf;
use std::thread;
use std::time::Duration;

impl Torrent {
    /// Download torrent.
    ///
    /// # Arguments
    ///
    /// * `filepath` - Path where to save the file.
    ///
    pub fn download(&self, filepath: PathBuf) -> Result<()> {
        let peers = self.peers.to_owned();

        // Create a work channel channel of unbounded capacity
        let work_chan: (Sender<PieceWork>, Receiver<PieceWork>) = unbounded();

        // Create a result channel of unbounded capacity
        let result_chan: (Sender<PieceResult>, Receiver<PieceResult>) = unbounded();

        // Init workers
        for index in 0..self.pieces_hashes.len() {
            // Create work piece
            let hash = self.pieces_hashes[index].clone();
            let length = self.piece_length;
            let piece_work = PieceWork::new(index, hash, length);

            // Send piece to work channel
            work_chan.0.send(piece_work)?;
        }

        // Start workers
        for peer in peers {
            let self_copy = self.clone();
            let work_chan_copy = work_chan.clone();
            let result_chan_copy = result_chan.clone();
            thread::spawn(move || {
                self_copy.start_worker(peer, work_chan_copy, result_chan_copy);
            });
        }

        while 1 == 1 {}

        Ok(())
    }

    /// Start worker.
    ///
    /// # Arguments
    ///
    /// * `peer` - The remote peer.
    /// * `work_chan` - The channel to send and receive work pieces.
    /// * `result_chan` - The channel to send result pieces.
    ///
    fn start_worker(
        &self,
        peer: Peer,
        work_chan: (Sender<PieceWork>, Receiver<PieceWork>),
        result_chan: (Sender<PieceResult>, Receiver<PieceResult>),
    ) {
        let peer_copy = peer.clone();
        let peer_id_copy = self.peer_id.clone();
        let info_hash_copy = self.info_hash.clone();

        // Create new client
        let mut client = match Client::new(peer_copy, peer_id_copy, info_hash_copy) {
            Ok(client) => client,
            Err(_) => return,
        };

        // Handshake with peer
        if client.handshake_with_peer().is_err() {
            return;
        }

        // Read bitfield from peer
        let mut bitfield: Bitfield = match client.read_bitfield() {
            Ok(msg) => msg.get_payload(),
            Err(_) => return,
        };

        // Send unchoke
        if client.send_unchoke().is_err() {
            return;
        }

        // Send interested
        if client.send_interested().is_err() {
            return;
        }

        println!("Connected to peer {:?}:{:?}", &peer.ip, &peer.port);

        loop {
            // Receive a piece from work channel
            let piece_work: PieceWork = match work_chan.1.recv() {
                Ok(piece_work) => piece_work,
                Err(_) => return,
            };

            // Check if peer has piece
            if !has_piece(&mut bitfield, piece_work.get_index()) {
                // Resend piece to work channel
                if work_chan.0.send(piece_work).is_err() {
                    println!("Error: could not send work piece to channel")
                }
                return;
            }

            println!("Downloading piece {:?}", piece_work.get_index());

            // Download piece
            let buf = match client.download_piece(piece_work) {
                Ok(buf) => buf,
                Err(_) => return,
            };
        }
    }
}

impl Client {
    /// Download a piece.
    ///
    /// # Arguments
    ///
    /// * `piece_work` - A work piece.
    ///
    fn download_piece(&self, piece_work: PieceWork) -> Result<Vec<u8>> {
        // Set connection timeout
        self.set_connection_timeout(30)?;

        // Download torrent piece
        while piece_work.get_downloaded() < piece_work.get_length() {
            thread::sleep(Duration::from_secs(1));
        }

        Ok(piece_work.get_buf())
    }
}
