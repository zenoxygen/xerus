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

/// PieceWork structure.
#[derive(Default, Debug, Clone)]
pub struct PieceWork {
    // Piece index
    index: u32,
    // Piece hash
    hash: Vec<u8>,
    // Piece length
    length: u32,
    // Piece data
    data: Vec<u8>,
    // Requested counter in bytes
    requested: u32,
    // Downloaded counter in bytes
    downloaded: u32,
}

/// PieceResult structure.
#[derive(Default, Debug, Clone)]
pub struct PieceResult {
    // Piece index
    index: u32,
    // Piece data
    data: Vec<u8>,
}

impl PieceWork {
    /// Build a new piece.
    ///
    /// # Arguments
    ///
    /// * `index` - The piece index.
    /// * `hash` - The piece hash.
    /// * `length` - The piece length.
    ///
    pub fn new(index: u32, hash: Vec<u8>, length: u32) -> Result<PieceWork> {
        let piece_work = PieceWork {
            index,
            hash,
            length,
            data: vec![0; length as usize],
            requested: 0,
            downloaded: 0,
        };

        Ok(piece_work)
    }

    /// Get work piece index.
    pub fn get_index(&self) -> u32 {
        self.index
    }

    /// Get work piece length.
    pub fn get_length(&self) -> u32 {
        self.length
    }

    /// Get work piece data.
    pub fn get_data(&self) -> Vec<u8> {
        self.data.to_vec()
    }

    /// Get work piece requested counter.
    pub fn get_requested(&self) -> u32 {
        self.requested
    }

    /// Set work piece requested counter.
    pub fn set_requested(&mut self, requested: u32) {
        self.requested = requested
    }

    /// Get work piece downloaded counter.
    pub fn get_downloaded(&self) -> u32 {
        self.downloaded
    }

    /// Set work piece downloaded counter.
    pub fn set_downloaded(&mut self, downloaded: u32) {
        self.downloaded = downloaded
    }
}