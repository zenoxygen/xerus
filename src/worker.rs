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
use crate::message::*;
use crate::peer::*;
use crate::piece::*;

use anyhow::{anyhow, Result};
use crossbeam_channel::{Receiver, Sender};
use crypto::digest::Digest;
use crypto::sha1::Sha1;

// Maximum number of requests
const NB_REQUESTS_MAX: u32 = 5;

// Block size limit (2^14) in bytes
const BLOCK_SIZE_MAX: u32 = 16384;

pub struct Worker {
    peer: Peer,
    peer_id: Vec<u8>,
    info_hash: Vec<u8>,
    work_chan: (Sender<PieceWork>, Receiver<PieceWork>),
    result_chan: (Sender<PieceResult>, Receiver<PieceResult>),
}

impl Worker {
    /// Build a new worker.
    ///
    /// # Arguments
    ///
    /// * `peer` - A remote peer to connect to.
    /// * `work_chan` - The channel to send and receive work pieces.
    /// * `result_chan` - The channel to send result pieces.
    ///
    pub fn new(
        peer: Peer,
        peer_id: Vec<u8>,
        info_hash: Vec<u8>,
        work_chan: (Sender<PieceWork>, Receiver<PieceWork>),
        result_chan: (Sender<PieceResult>, Receiver<PieceResult>),
    ) -> Result<Worker> {
        // Create a new worker
        let worker = Worker {
            peer,
            peer_id,
            info_hash,
            work_chan,
            result_chan,
        };

        Ok(worker)
    }

    /// Start worker.
    pub fn start_download(&self) {
        let peer_copy = self.peer.clone();
        let peer_id_copy = self.peer_id.clone();
        let info_hash_copy = self.info_hash.clone();

        // Create new client
        let mut client = match Client::new(peer_copy, peer_id_copy, info_hash_copy) {
            Ok(client) => client,
            Err(_) => return,
        };

        // Set connection timeout
        if client.set_connection_timeout(5).is_err() {
            return;
        }

        // Handshake with peer
        if client.handshake_with_peer().is_err() {
            return;
        }

        // Read bitfield from peer
        if client.read_bitfield().is_err() {
            return;
        }

        // Send unchoke
        if client.send_unchoke().is_err() {
            return;
        }

        // Send interested
        if client.send_interested().is_err() {
            return;
        }

        loop {
            // Receive a piece from work channel
            let mut piece_work: PieceWork = match self.work_chan.1.recv() {
                Ok(piece_work) => piece_work,
                Err(_) => {
                    println!("Error: could not receive piece from channel");
                    return;
                }
            };

            // Check if remote peer has piece
            if !client.has_piece(piece_work.index) {
                // Resend piece to work channel
                if self.work_chan.0.send(piece_work).is_err() {
                    println!("Error: could not send piece to channel")
                }
                continue;
            }

            // Download piece
            if self.download_piece(&mut client, &mut piece_work).is_err() {
                // Resend piece to work channel
                if self.work_chan.0.send(piece_work).is_err() {
                    println!("Error: could not send piece to channel")
                }
                return;
            }

            // Verify piece integrity
            if self.verify_piece_integrity(&mut piece_work).is_err() {
                // Resend piece to work channel
                if self.work_chan.0.send(piece_work).is_err() {
                    println!("Error: could not send piece to channel")
                }
                continue;
            }

            // Notify peer that piece was downloaded
            if client.send_have(piece_work.index).is_err() {
                println!("Error: could not notify peer that piece was downloaded");
                return;
            }

            // Send piece to result channel
            let piece_result = PieceResult::new(piece_work.index, piece_work.data);
            if self.result_chan.0.send(piece_result).is_err() {
                println!("Error: could not send piece to channel")
            }
        }
    }

    /// Download a torrent piece.
    ///
    /// # Arguments
    ///
    /// * `client` - A client connected to a remote peer.
    /// * `piece_work` - A piece to download.
    ///
    fn download_piece(&self, client: &mut Client, piece_work: &mut PieceWork) -> Result<()> {
        // Set client connection timeout
        client.set_connection_timeout(30)?;

        // Download torrent piece
        while piece_work.downloaded < piece_work.length {
            // If client is unchoked by peer
            if !client.is_choked() {
                while piece_work.requests < NB_REQUESTS_MAX
                    && piece_work.requested < piece_work.length
                {
                    // Get block size to request
                    let mut block_size = BLOCK_SIZE_MAX;
                    let remaining = piece_work.length - piece_work.requested;
                    if remaining < BLOCK_SIZE_MAX {
                        block_size = remaining;
                    }

                    // Send request for a block
                    client.send_request(piece_work.index, piece_work.requested, block_size)?;

                    // Update number of requests sent
                    piece_work.requests += 1;

                    // Update size of requested data
                    piece_work.requested += block_size;
                }
            }

            // Listen peer
            let message: Message = client.read_message()?;

            // Parse message
            match message.get_id() {
                MESSAGE_CHOKE => client.read_choke(),
                MESSAGE_UNCHOKE => client.read_unchoke(),
                MESSAGE_HAVE => client.read_have(message)?,
                MESSAGE_PIECE => client.read_piece(message, piece_work)?,
                _ => println!("received unknown message from peer"),
            }
        }
        Ok(())
    }

    /// Verify the integrity of a downloaded torrent piece.
    ///
    /// # Arguments
    ///
    /// * `piece_work` - A piece to download.
    ///
    fn verify_piece_integrity(&self, piece_work: &mut PieceWork) -> Result<()> {
        // Hash piece data
        let mut hasher = Sha1::new();
        hasher.input(&piece_work.data);

        // Read hash digest
        let hex = hasher.result_str();

        // Decoded hex string into bytes
        let decoded: Vec<u8> = hex::decode(hex)?;

        // Compare hashes
        if decoded != piece_work.hash {
            return Err(anyhow!(
                "could not verify integrity of piece downloaded from peer"
            ));
        }

        println!(
            "Successfully verified integrity of piece {:?}",
            piece_work.index
        );

        Ok(())
    }
}
