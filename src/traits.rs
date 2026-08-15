use teloxide::payloads::{
    SendMessageSetters, SendPhotoSetters, SendStickerSetters, SendVoiceSetters,
};
use teloxide::types::{
    ChatKind, LinkPreviewOptions, Message, MessageId, PublicChatKind,
    ReplyParameters, ThreadId,
};

#[derive(Clone, Copy)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub enum ThreadTarget {
    None,
    Thread(ThreadId),
    /// Topic too far back for the Bot API — reply to the trigger instead.
    ReplyTo(MessageId),
}

pub fn thread_target(m: &Message) -> ThreadTarget {
    let is_forum = match &m.chat.kind {
        ChatKind::Public(p) => match &p.kind {
            PublicChatKind::Supergroup(s) => s.is_forum,
            _ => false,
        },
        _ => false,
    };

    if !m.is_topic_message || !is_forum {
        return ThreadTarget::None;
    }

    let Some(thread_id) = m.thread_id else {
        return ThreadTarget::None;
    };

    // When message from thread_id is not visible for Bot API (1 m messages
    // back) - force reply to message in thread
    if thread_id.0.0 < (m.id.0 - 1000000) {
        return ThreadTarget::ReplyTo(m.id);
    }

    ThreadTarget::Thread(thread_id)
}

macro_rules! define_maybe_setter {
    ($setter:ident, $trait:ident) => {
        pub trait $trait {
            fn maybe_thread_id(self, m: &Message) -> Self;
        }

        impl<T: $setter> $trait for T {
            fn maybe_thread_id(self, m: &Message) -> Self {
                match $crate::traits::thread_target(m) {
                    ThreadTarget::None => self,
                    ThreadTarget::ReplyTo(id) => {
                        self.reply_parameters(ReplyParameters::new(id))
                    },
                    ThreadTarget::Thread(thread_id) => {
                        self.message_thread_id(thread_id)
                    },
                }
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

#[cfg(test)]
mod tests {
    use super::*;
    use teloxide::types::MessageId;

    /// A group/supergroup message, deserialized the way Telegram sends it.
    fn message(
        chat_json: &str,
        message_id: i32,
        thread_id: Option<i32>,
        is_topic: bool,
    ) -> Message {
        let thread = thread_id
            .map(|t| format!(r#""message_thread_id": {t},"#))
            .unwrap_or_default();

        let json = format!(
            r#"{{
                "message_id": {message_id},
                {thread}
                "is_topic_message": {is_topic},
                "date": 1753700000,
                "chat": {chat_json},
                "text": "hi"
            }}"#
        );

        serde_json::from_str(&json).expect("bad Message fixture")
    }

    const FORUM: &str = r#"{
        "id": -1001234567890,
        "type": "supergroup",
        "title": "Forum",
        "is_forum": true
    }"#;

    const SUPERGROUP: &str = r#"{
        "id": -1001234567890,
        "type": "supergroup",
        "title": "Plain"
    }"#;

    const GROUP: &str =
        r#"{ "id": -100, "type": "group", "title": "Old group" }"#;

    const PRIVATE: &str = r#"{ "id": 42, "type": "private" }"#;

    #[test]
    fn a_topic_message_in_a_forum_targets_its_thread() {
        let m = message(FORUM, 2_000, Some(1_500), true);

        assert_eq!(
            thread_target(&m),
            ThreadTarget::Thread(teloxide::types::ThreadId(MessageId(1_500)))
        );
    }

    #[test]
    fn an_old_thread_falls_back_to_replying_to_the_trigger() {
        // The Bot API cannot resolve a topic opened more than ~1M messages
        // ago, so the reply is anchored to the incoming message instead.
        let m = message(FORUM, 2_000_000, Some(5), true);

        assert_eq!(thread_target(&m), ThreadTarget::ReplyTo(MessageId(2_000_000)));
    }

    #[test]
    fn the_one_million_boundary_is_exclusive() {
        // thread_id < message_id - 1_000_000 switches to the reply form.
        let just_inside = message(FORUM, 2_000_000, Some(1_000_000), true);
        assert!(matches!(
            thread_target(&just_inside),
            ThreadTarget::Thread(_)
        ));

        let just_outside = message(FORUM, 2_000_000, Some(999_999), true);
        assert!(matches!(
            thread_target(&just_outside),
            ThreadTarget::ReplyTo(_)
        ));
    }

    #[test]
    fn a_non_topic_message_in_a_forum_is_sent_plainly() {
        let m = message(FORUM, 2_000, Some(1_500), false);
        assert_eq!(thread_target(&m), ThreadTarget::None);
    }

    #[test]
    fn a_topic_flag_outside_a_forum_is_ignored() {
        for chat in [SUPERGROUP, GROUP, PRIVATE] {
            let m = message(chat, 2_000, Some(1_500), true);
            assert_eq!(thread_target(&m), ThreadTarget::None, "{chat}");
        }
    }

    #[test]
    fn a_topic_message_without_a_thread_id_is_sent_plainly() {
        let m = message(FORUM, 2_000, None, true);
        assert_eq!(thread_target(&m), ThreadTarget::None);
    }
}
