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

/// A Bitfield represents the pieces a peer has.
pub type Bitfield = Vec<u8>;

/// Check if bitfield has a piece index.
pub fn has_piece(bitfield: &mut Bitfield, index: usize) -> bool {
    let byte_index = index / 8;
    let offset = index % 8;
    let bitfield_len = bitfield.len() as usize;

    // Prevent unbounded values
    if byte_index >= bitfield_len {
        false
    } else {
        bitfield[byte_index] >> (7 - offset) as u8 & 1 != 0
    }
}

/// Set a piece into bitfield.
pub fn set_piece(bitfield: &Bitfield, index: usize) -> Bitfield {
    let byte_index = index / 8;
    let offset = index % 8;
    let bitfield_len = bitfield.len() as usize;
    let mut new_bitfield = bitfield.to_vec();

    // Prevent unbounded values
    if byte_index >= bitfield_len {
        new_bitfield
    } else {
        new_bitfield[byte_index] |= (1 << (7 - offset)) as u8;
        new_bitfield
    }
}
