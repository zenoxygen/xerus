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
use byteorder::{BigEndian, WriteBytesExt};

type MessageId = u8;
type MessagePayload = Vec<u8>;

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

#[derive(Default, Debug)]
pub struct Message {
    // Message type
    id: MessageId,
    // Message payload
    payload: MessagePayload,
}

impl Message {
    /// Build a new message.
    ///
    /// # Arguments
    ///
    /// * `id` - The type of the message.
    ///
    pub fn new(id: MessageId) -> Self {
        Message {
            id,
            payload: vec![],
        }
    }

    /// Build a new message with a payload.
    ///
    /// # Arguments
    ///
    /// * `id` - The type of the message.
    /// * `payload` - The content of the message.
    ///
    pub fn new_with_payload(id: MessageId, payload: MessagePayload) -> Self {
        Message { id, payload }
    }

    /// Get message id.
    pub fn get_id(&self) -> MessageId {
        self.id
    }

    /// Get message payload.
    pub fn get_payload(&self) -> MessagePayload {
        self.payload.to_vec()
    }

    /// Serialize message.
    pub fn serialize(&self) -> Result<Vec<u8>> {
        // Get message length
        let message_len = 1 + self.payload.len();

        // Create a new buffer
        let mut serialized: Vec<u8> = vec![0; 4 + message_len];

        // Add message length
        let mut len: Vec<u8> = vec![];
        len.write_u32::<BigEndian>(message_len as u32)?;
        serialized.append(&mut len);

        // Add message id
        serialized.push(self.id);

        // Add message payload
        let mut payload = self.payload.clone();
        serialized.append(&mut payload);

        Ok(serialized)
    }
}

/// Deserialize message.
///
/// # Arguments
///
/// * `message_buf` - The message to deserialize.
/// * `message_len` - The message length.
///
pub fn deserialize_message(message_buf: &Vec<u8>, message_len: usize) -> Result<Message> {
    // Get message id
    let id: MessageId = message_buf[0];

    // Get message payload
    let mut payload: MessagePayload = vec![];
    for (i, x) in message_buf.iter().enumerate() {
        if i > 0 && i < message_len {
            payload.push(x.to_owned());
        }
    }

    // Build message
    let message = Message::new_with_payload(id, payload);

    Ok(message)
}
