use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use packed_struct::prelude::*;

const ID32_FORMAT_SIZE: usize = 27;

#[derive(PackedStruct, Debug)]
pub struct MessageData32 {
    #[packed_field(endian = "lsb")]
    dc_id: i32,
    #[packed_field(endian = "lsb")]
    message_id: i32,
    #[packed_field(endian = "lsb")]
    chat_id: i32,
    #[packed_field(endian = "lsb")]
    access_hash: i64,
}

#[derive(PackedStruct, Debug)]
pub struct MessageData {
    #[packed_field(endian = "lsb")]
    pub dc_id: i32,
    #[packed_field(endian = "lsb")]
    pub chat_id: i64,
    #[packed_field(endian = "lsb")]
    pub message_id: i32,
    #[packed_field(endian = "lsb")]
    pub access_hash: i64,
}

pub fn decode_inline_message_id(
    inline_message_id: &str,
) -> Option<MessageData> {
    let decoded_inline_message_id =
        URL_SAFE_NO_PAD.decode(inline_message_id).ok()?;

    if inline_message_id.len() == ID32_FORMAT_SIZE {
        let data = MessageData32::unpack_from_slice(&decoded_inline_message_id)
            .ok()?;
        // Force message data for simplicity
        MessageData {
            dc_id: data.dc_id,
            chat_id: data.chat_id as i64,
            message_id: data.message_id,
            access_hash: data.access_hash,
        }
    } else {
        MessageData::unpack_from_slice(&decoded_inline_message_id).ok()?
    }
    .into()
}

// https://github.com/teloxide/teloxide/blob/ae0451f7d72e78fdeb317db397fb602a29eda17a/crates/teloxide-core/src/types/chat_id.rs#L107
const MAX_MARKED_CHANNEL_ID: i64 = -1000000000000;

impl MessageData {
    // Normalize MessageData to normal Bot API fields
    // - chat_id from -* to -100*
    pub fn normalize(&mut self) {
        // If chat_id is ige
        if self.chat_id.is_negative() {
            self.chat_id += MAX_MARKED_CHANNEL_ID;
        };
    }
}
