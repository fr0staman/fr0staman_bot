use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use byteorder::{LittleEndian, ReadBytesExt};

const ID32_FORMAT_SIZE: usize = 27;

#[allow(dead_code)]
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
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
    // Idempotent: applying the offset twice yields an id for no chat at all.
    pub fn normalize(&mut self) {
        if self.chat_id.is_negative() && !self.is_normalized() {
            self.chat_id += MAX_MARKED_CHANNEL_ID;
        };
    }

    pub fn is_normalized(&self) -> bool {
        self.chat_id <= MAX_MARKED_CHANNEL_ID
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds an `inline_message_id` the way Telegram's TL serializer would.
    fn encode(
        dc_id: i32,
        message_id: i32,
        chat_id: i64,
        access_hash: i64,
        i32_chat: bool,
    ) -> String {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&dc_id.to_le_bytes());
        bytes.extend_from_slice(&message_id.to_le_bytes());
        if i32_chat {
            bytes.extend_from_slice(&(chat_id as i32).to_le_bytes());
        } else {
            bytes.extend_from_slice(&chat_id.to_le_bytes());
        }
        bytes.extend_from_slice(&access_hash.to_le_bytes());

        URL_SAFE_NO_PAD.encode(bytes)
    }

    #[test]
    fn decodes_the_i32_chat_form() {
        // 20 bytes -> 27 base64 chars, which is what selects the i32 branch.
        let encoded = encode(2, 4242, -1_234_567, -9_000_000_000, true);
        assert_eq!(encoded.len(), ID32_FORMAT_SIZE);

        assert_eq!(
            decode_inline_message_id(&encoded),
            Some(MessageData {
                dc_id: 2,
                chat_id: -1_234_567,
                message_id: 4242,
                access_hash: -9_000_000_000,
            })
        );
    }

    #[test]
    fn decodes_the_i64_chat_form() {
        let encoded =
            encode(5, 77, -1_001_234_567_890, 1_234_567_890_123, false);
        assert_ne!(encoded.len(), ID32_FORMAT_SIZE);

        assert_eq!(
            decode_inline_message_id(&encoded),
            Some(MessageData {
                dc_id: 5,
                chat_id: -1_001_234_567_890,
                message_id: 77,
                access_hash: 1_234_567_890_123,
            })
        );
    }

    #[test]
    fn rejects_malformed_base64() {
        for bad in ["", "!!!!", "not base64 at all", "===="] {
            assert_eq!(decode_inline_message_id(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn rejects_a_truncated_buffer() {
        // Valid base64, but too few bytes for all four fields.
        let short = URL_SAFE_NO_PAD.encode([1u8, 2, 3, 4, 5, 6]);
        assert_eq!(decode_inline_message_id(&short), None);
    }

    #[test]
    fn a_27_char_id_is_always_read_as_an_i32_chat() {
        // The branch keys off the *string length*, not the payload, so a
        // 27-char id is read as i32 even when the bytes came from an i64.
        let encoded = encode(1, 1, 1, 1, true);
        assert_eq!(encoded.len(), ID32_FORMAT_SIZE);
        assert_eq!(decode_inline_message_id(&encoded).unwrap().chat_id, 1);
    }


    #[test]
    fn normalize_marks_a_negative_chat_id_as_a_supergroup() {
        let mut data = MessageData {
            dc_id: 1,
            chat_id: -1_234_567,
            message_id: 1,
            access_hash: 1,
        };
        data.normalize();

        assert_eq!(data.chat_id, -1_000_001_234_567);
    }

    #[test]
    fn normalize_leaves_a_positive_chat_id_alone() {
        let mut data = MessageData {
            dc_id: 1,
            chat_id: 555,
            message_id: 1,
            access_hash: 1,
        };
        data.normalize();

        assert_eq!(data.chat_id, 555);
    }

    #[test]
    fn normalize_is_idempotent() {
        // Calling it twice used to subtract the marker twice and produce an
        // id belonging to no chat at all.
        let mut data = MessageData {
            dc_id: 1,
            chat_id: -1,
            message_id: 1,
            access_hash: 1,
        };

        data.normalize();
        assert_eq!(data.chat_id, -1_000_000_000_001);
        assert!(data.is_normalized());

        for _ in 0..5 {
            data.normalize();
            assert_eq!(data.chat_id, -1_000_000_000_001);
        }
    }

    #[test]
    fn an_already_marked_id_is_left_alone() {
        // Telegram sometimes hands back an id that is already in the Bot API
        // form; re-marking it would corrupt it.
        let mut data = MessageData {
            dc_id: 1,
            chat_id: -1_001_234_567_890,
            message_id: 1,
            access_hash: 1,
        };

        assert!(data.is_normalized());
        data.normalize();
        assert_eq!(data.chat_id, -1_001_234_567_890);
    }

    #[test]
    fn normalizing_never_moves_an_id_out_of_the_marked_range() {
        for raw in [-1_i64, -999, -1_234_567, -999_999_999_999] {
            let mut data = MessageData {
                dc_id: 1,
                chat_id: raw,
                message_id: 1,
                access_hash: 1,
            };

            data.normalize();
            assert!(data.is_normalized(), "{raw} -> {}", data.chat_id);

            let once = data.chat_id;
            data.normalize();
            assert_eq!(data.chat_id, once, "{raw} was normalized twice");
        }
    }
}
