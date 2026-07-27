//! Quake Companion — Tauri v2 app for the Decokee QUAKE display panel.
//!
//! The crate is split into a Tauri-free core (`hid`, `device`, `stats`) and an
//! optional Tauri shell (`commands`, `app`, gated on the `tauri-runtime`
//! feature). The core compiles on hosts that lack the webkit2gtk system
//! libraries, so the protocol + device logic can be type-checked in isolation.

pub mod config;
pub mod device;
pub mod hid;
pub mod stats;
pub mod via;

#[cfg(feature = "tauri-runtime")]
pub mod app;
#[cfg(feature = "tauri-runtime")]
pub mod commands;