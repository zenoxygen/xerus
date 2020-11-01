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
use byteorder::{BigEndian, ByteOrder};

type MessageId = u8;

pub const MESSAGE_CHOKE: MessageId = 0;
pub const MESSAGE_UNCHOKE: MessageId = 1;
pub const MESSAGE_INTERESTED: MessageId = 2;
pub const MESSAGE_NOT_INTERESTED: MessageId = 3;
pub const MESSAGE_HAVE: MessageId = 4;
pub const MESSAGE_BITFIELD: MessageId = 5;
pub const MESSAGE_REQUEST: MessageId = 6;
pub const MESSAGE_PIECE: MessageId = 7;
pub const MESSAGE_CANCEL: MessageId = 8;
pub const MESSAGE_PORT: MessageId = 9;

#[derive(Default)]
pub struct Message {
    // Message type
    pub id: MessageId,
    // Message content
    pub payload: Vec<u8>,
}

impl Message {
    /// Build a new message.
    ///
    /// # Arguments
    ///
    /// * `id` - The type of the message.
    ///
    pub fn new(id: u8) -> Result<Message> {
        let payload: Vec<u8> = vec![];

        // Build message
        let message = Message { id, payload };

        Ok(message)
    }

    /// Build a new message with a payload.
    ///
    /// # Arguments
    ///
    /// * `id` - The type of the message.
    /// * `payload` - The content of the message.
    ///
    pub fn new_with_payload(id: u8, payload: Vec<u8>) -> Result<Message> {
        // Build message
        let message = Message { id, payload };

        Ok(message)
    }

    /// Serialize message.
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let message_len = 1 + self.payload.len();
        let mut serialized: Vec<u8> = vec![0; 4 + message_len];

        // Add length
        let mut buf = [0; 4];
        BigEndian::write_u32(&mut buf, message_len as u32);
        serialized.append(&mut buf.to_vec());

        // Add id
        serialized.push(self.id);

        // Add payload
        let mut payload = self.payload.clone();
        serialized.append(&mut payload);

        Ok(serialized)
    }
}

/// Deserialize message.
pub fn deserialize_message(buf_message: &Vec<u8>, len: u32) -> Result<Message> {
    // Get id
    let id: u8 = buf_message[0];

    // Get payload
    let mut payload = Vec::new();
    for (i, x) in buf_message.iter().enumerate() {
        if i > 0 && i < (len as usize) {
            payload.push(x.to_owned());
        }
    }

    // Build message
    let message = Message::new_with_payload(id, payload)?;

    Ok(message)
}
