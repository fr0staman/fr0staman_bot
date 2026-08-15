use teloxide::{
    dispatching::{UpdateFilterExt, UpdateHandler},
    prelude::*,
    types::MessageKind,
};

use crate::{
    enums::{AdminCommands, EpycCommands, MyCommands},
    handlers::{
        admin, callback, command, epyc, feedback, inline, message, system,
    },
    types::MyError,
};

/// `creator_id` is a parameter rather than a `BOT_CONFIG` read, so building
/// the tree needs no globals.
pub fn build_handler(creator_id: u64) -> UpdateHandler<MyError> {
    dptree::entry()
        .branch(
            Update::filter_message()
                .branch(
                    dptree::entry()
                        .filter_command::<MyCommands>()
                        .endpoint(command::filter_commands),
                )
                .branch(
                    dptree::entry()
                        .filter_command::<EpycCommands>()
                        .endpoint(epyc::filter_commands),
                )
                .branch(
                    dptree::filter(move |m: Message| {
                        m.from.as_ref().is_some_and(|u| u.id.0 == creator_id)
                    })
                    .filter_command::<AdminCommands>()
                    .endpoint(admin::filter_admin_commands),
                )
                .branch(
                    Message::filter_new_chat_members()
                        .endpoint(system::handle_new_member),
                )
                .branch(
                    Message::filter_left_chat_member()
                        .endpoint(system::handle_left_member),
                )
                .branch(
                    dptree::filter(|m: Message| {
                        m.migrate_to_chat_id().is_some()
                    })
                    .endpoint(system::handle_chat_migration),
                )
                .branch(
                    dptree::filter(|m: Message| {
                        matches!(
                            m.kind,
                            MessageKind::VideoChatStarted(_)
                                | MessageKind::VideoChatEnded(_)
                        )
                    })
                    .endpoint(system::handle_video_chat),
                )
                .branch(
                    dptree::filter(|m: Message| m.text().is_some())
                        .endpoint(message::handle_message),
                )
                .branch(
                    dptree::filter(|m: Message| {
                        m.voice().is_some() && m.chat.is_private()
                    })
                    .endpoint(system::handle_voice_private),
                )
                .branch(
                    dptree::filter(|m: Message| {
                        m.animation().is_some() && m.chat.is_private()
                    })
                    .endpoint(system::handle_animation_private),
                ),
        )
        .branch(
            Update::filter_inline_query()
                .endpoint(inline::filter_inline_commands),
        )
        .branch(
            Update::filter_callback_query()
                .endpoint(callback::filter_callback_commands),
        )
        .branch(
            Update::filter_my_chat_member()
                .filter(|u: Update| u.chat().is_some_and(|c| c.is_private()))
                .endpoint(system::handle_ban_or_unban_in_private),
        )
        .branch(
            Update::filter_chosen_inline_result()
                .endpoint(feedback::filter_inline_feedback_commands),
        )
}
