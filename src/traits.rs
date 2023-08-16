use teloxide::payloads::{
    SendMessageSetters, SendPhotoSetters, SendStickerSetters, SendVoiceSetters,
};
use teloxide::types::{ChatKind, Message, PublicChatKind};

macro_rules! define_maybe_setter {
    ($setter:ident, $trait:ident) => {
        pub trait $trait {
            fn maybe_thread_id(self, m: &Message) -> Self;
        }

        impl<T: $setter> $trait for T {
            fn maybe_thread_id(self, m: &Message) -> Self {
                if let Some(thread_id) = m.thread_id {
                    let is_forum = match &m.chat.kind {
                        ChatKind::Public(p) => match &p.kind {
                            PublicChatKind::Supergroup(s) => s.is_forum,
                            _ => false,
                        },
                        _ => false,
                    };
                    if is_forum {
                        self.message_thread_id(thread_id)
                            .allow_sending_without_reply(true)
                    } else {
                        self
                    }
                } else {
                    self
                }
            }
        }
    };
}

define_maybe_setter!(SendMessageSetters, MaybeMessageSetter);
define_maybe_setter!(SendStickerSetters, MaybeStickerSetter);
define_maybe_setter!(SendVoiceSetters, MaybeVoiceSetter);
define_maybe_setter!(SendPhotoSetters, MaybePhotoSetter);
