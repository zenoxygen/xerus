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

/// PieceWork structure.
#[derive(Default, Debug, Clone)]
pub struct PieceWork {
    // Piece index
    pub index: u32,
    // Piece hash
    pub hash: Vec<u8>,
    // Piece length
    pub length: u32,
    // Piece data
    pub data: Vec<u8>,
    // Requests number sent
    pub requests: u32,
    // Size of requested data in bytes
    pub requested: u32,
    // Size of downloaded data in bytes
    pub downloaded: u32,
}

/// PieceResult structure.
#[derive(Default, Debug, Clone)]
pub struct PieceResult {
    // Piece index
    pub index: u32,
    // Piece data
    pub data: Vec<u8>,
}

impl PieceWork {
    /// Build a new work piece.
    ///
    /// # Arguments
    ///
    /// * `index` - The piece index.
    /// * `hash` - The piece hash.
    /// * `length` - The piece length.
    ///
    pub fn new(index: u32, hash: Vec<u8>, length: u32) -> PieceWork {
        PieceWork {
            index,
            hash,
            length,
            data: vec![0; length as usize],
            requests: 0,
            requested: 0,
            downloaded: 0,
        }
    }
}

impl PieceResult {
    /// Build a new result piece.
    ///
    /// # Arguments
    ///
    /// * `index` - The piece index.
    /// * `data` - The piece data.
    ///
    pub fn new(index: u32, data: Vec<u8>) -> PieceResult {
        PieceResult { index, data }
    }
}
