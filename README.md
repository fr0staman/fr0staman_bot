<div align="center">
  <h1>fr0staman_bot</h1>
  <p>
    <strong>Multifunctional funny Telegram bot with pigs 🐷</strong>
  </p>
  <p>

![MSRV](https://img.shields.io/badge/rustc-1.80+-ab6000.svg)
[![](https://img.shields.io/badge/Telegram-bot-blue?logo=telegram)](https://t.me/fr0staman_bot)
[![](https://img.shields.io/badge/Telegram-chat-blue?logo=telegram)](https://t.me/fr0staman_chat)

  </p>
</div>

## User manual:

- [🇺🇦 Ukrainian](https://telegra.ph/Help--fr0staman-bot-uk-08-05)
- [English](https://telegra.ph/Help--fr0staman-bot-en-08-05)
- [russian](https://telegra.ph/Help--fr0staman-bot-ru-08-05)

## Overview
❗️ This repository is rather not an example for beginners, as it combines several technologies, which can complicate the initial learning path.

**@fr0staman_bot** is a multifunctional game-like bot written in [Rust](https://www.rust-lang.org/), using [teloxide](https://github.com/teloxide/teloxide).
Bot uses much from Telegram Bot API and solves many (non) classic tasks, so in the code you can see:
- Multilingualism (with user or chat settings!)
- Message/Inline/Callback/Chosen filter and handling
- Optional new/left user reaction
- Handling chat_migration from chat to supergroup
- Other Telegram events (video_chat, chat_migration)
- Telegram channel subscription check
- Receive from user, moderating gif and voice bot inline content
- Storing and updating basic information about users and chats
- [Decode inline_message_id](https://github.com/fr0staman/fr0staman_bot/blob/master/src/utils/decode.rs) and [join inline chats with supergroups](https://github.com/fr0staman/fr0staman_bot/blob/master/src/handlers/callback.rs#L1160)
- Callback chained locking per user
- Increase voice message volume with `libopus`
- Quite detailed logging
- Sending bot errors directly to Telegram log group
- Metrics with [Prometheus](https://prometheus.io/)

## Deployment
1. Install `diesel_cli` (`--no-default-features --features=mysql`)
2. Install and create `mysql` database
3. Copy and fill `.env` from `.env.example`
4. Setup diesel migrations by `diesel migration run`
5. Build and start bot (`cargo build --release && target/release/fr0staman_bot`, `cargo run --release`)
6. Enjoy 🐽
