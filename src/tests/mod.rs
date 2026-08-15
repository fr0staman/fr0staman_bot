//! Tests that span several modules, kept together rather than split across
//! the code they exercise.
//!
//! These were `tests/*.rs` integration tests. As in-crate `#[cfg(test)]`
//! modules they compile into the single test binary instead of nine separate
//! ones, each of which had to link the whole dependency graph.

mod codecs;
mod commands;
mod keyboards;
mod locales;

mod common;
mod db_chat_pig;
mod db_hand_pig;
mod db_other;
mod db_schema;
mod db_services;
