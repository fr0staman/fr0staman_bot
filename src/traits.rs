use teloxide::payloads::{
    SendMessageSetters, SendPhotoSetters, SendStickerSetters, SendVoiceSetters,
};
use teloxide::types::{ChatKind, LinkPreviewOptions, Message, PublicChatKind};

macro_rules! define_maybe_setter {
    ($setter:ident, $trait:ident) => {
        pub trait $trait {
            fn maybe_thread_id(self, m: &Message) -> Self;
        }

        impl<T: $setter> $trait for T {
            fn maybe_thread_id(self, m: &Message) -> Self {
                let is_forum = match &m.chat.kind {
                    ChatKind::Public(p) => match &p.kind {
                        PublicChatKind::Supergroup(s) => s.is_forum,
                        _ => false,
                    },
                    _ => false,
                };

                if !m.is_topic_message || !is_forum {
                    return self;
                }

                let Some(thread_id) = m.thread_id else {
                    return self;
                };

                self.message_thread_id(thread_id)
            }
        }
    };
}

pub trait SimpleDisableWebPagePreview {
    fn disable(preview: bool) -> LinkPreviewOptions;
}

impl SimpleDisableWebPagePreview for LinkPreviewOptions {
    fn disable(preview: bool) -> Self {
        LinkPreviewOptions {
            is_disabled: preview,
            url: None,
            prefer_small_media: false,
            prefer_large_media: false,
            show_above_text: false,
        }
    }
}

define_maybe_setter!(SendMessageSetters, MaybeMessageSetter);
define_maybe_setter!(SendStickerSetters, MaybeStickerSetter);
define_maybe_setter!(SendVoiceSetters, MaybeVoiceSetter);
define_maybe_setter!(SendPhotoSetters, MaybePhotoSetter);
