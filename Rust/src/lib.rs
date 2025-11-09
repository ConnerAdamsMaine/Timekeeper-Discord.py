use pyo3::prelude::*;

// Module declarations
mod _types;
mod enums;
mod errors;
mod utils;
mod backoff;
mod http;
mod gateway;
mod state;

// Core models
mod user;
mod guild;
mod channel;
mod message;
mod member;
mod role;
mod emoji;
mod permissions;
mod asset;
mod colour;
mod flags;
mod object;
mod mixins;

// Client and event handling
mod client;
mod shard;

// Event models
mod activity;
mod presences;
mod voice_state;
mod interactions;
mod components;
mod embeds;
mod mentions;
mod file;
mod sticker;
mod reaction;
mod poll;

// Other models
mod appinfo;
mod audit_logs;
mod automod;
mod integrations;
mod invite;
mod onboarding;
mod partial_emoji;
mod player;
mod primary_guild;
mod raw_models;
mod scheduled_event;
mod search;
mod sku;
mod soundboard;
mod stage_instance;
mod subscription;
mod team;
mod template;
mod threads;
mod webhook;
mod welcome_screen;
mod widget;
mod context_managers;

// Voice support
mod opus;
mod oggparse;
mod voice_client;

/// Discord.py reimplemented in Rust with Python bindings
#[pymodule]
fn discord(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Add version info
    m.add("__version__", "3.0.0-rust")?;

    // Register types module
    _types::register_module(m)?;

    // Register enums
    enums::register_module(m)?;

    // Register errors
    errors::register_module(m)?;

    // Register utility functions
    utils::register_module(m)?;

    // Register HTTP client
    http::register_module(m)?;

    // Register gateway
    gateway::register_module(m)?;

    // Register core models
    user::register_module(m)?;
    guild::register_module(m)?;
    channel::register_module(m)?;
    message::register_module(m)?;
    member::register_module(m)?;
    role::register_module(m)?;
    emoji::register_module(m)?;
    permissions::register_module(m)?;

    // Register client
    client::register_module(m)?;

    // Register state cache
    state::register_module(m)?;

    Ok(())
}
