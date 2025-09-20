use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use byteorder::{LittleEndian, ReadBytesExt};

const ID32_FORMAT_SIZE: usize = 27;

#[allow(dead_code)]
#[derive(Debug)]
pub struct MessageData {
    pub dc_id: i32,
    pub chat_id: i64,
    pub message_id: i32,
    pub access_hash: i64,
}

pub fn decode_inline_message_id(
    inline_message_id: &str,
) -> Option<MessageData> {
    let is_i32_chat = inline_message_id.len() == ID32_FORMAT_SIZE;
    let decoded_inline_message_id =
        URL_SAFE_NO_PAD.decode(inline_message_id).ok()?;

    let mut cursor = std::io::Cursor::new(decoded_inline_message_id);

    let dc_id = cursor.read_i32::<LittleEndian>().ok()?;
    let message_id = cursor.read_i32::<LittleEndian>().ok()?;
    let chat_id = if is_i32_chat {
        cursor.read_i32::<LittleEndian>().ok()? as i64
    } else {
        cursor.read_i64::<LittleEndian>().ok()?
    };
    let access_hash = cursor.read_i64::<LittleEndian>().ok()?;

    // Force message data for simplicity
    Some(MessageData { dc_id, chat_id, message_id, access_hash })
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
