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
1. Install `diesel_cli` (`--no-default-features --features=postgres`)
2. Install and create `postgres` database
3. Copy and fill `.env` from `.env.example`
4. Setup diesel migrations by `diesel migration run`
5. Build and start bot (`cargo build --release && target/release/fr0staman_bot`, `cargo run --release`)
6. Enjoy 🐽

## Testing

```bash
cargo test
```

Runs the pure unit tests — game formulas, achievement rules, duel resolution,
the callback and inline-result codecs, locale parity across all four
languages, command parsing. No database or network needed.

The database tests (`src/tests/db_*.rs`) are opt-in and skip silently
unless `TEST_DATABASE_URL` is set:

```bash
createdb fr0staman_test
export TEST_DATABASE_URL=postgres://user:pass@localhost/fr0staman_test
./scripts/setup_test_db.sh
cargo test
```

⚠️ The harness truncates every table before each test — point it at a scratch
database, never at your `DATABASE_URL`.

Some tests deliberately assert behaviour that looks wrong, so that fixing it
is a visible change rather than a silent one. They carry a `CHARACTERISATION`
or `BUG` comment; the full list is in [TESTING_FINDINGS.md](TESTING_FINDINGS.md).
