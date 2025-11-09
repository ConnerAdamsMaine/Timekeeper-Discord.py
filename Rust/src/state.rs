use pyo3::prelude::*;
use dashmap::DashMap;
use std::sync::Arc;
use crate::_types::Snowflake;

/// Connection state that caches Discord entities
pub struct State {
    // Cached guilds
    guilds: Arc<DashMap<Snowflake, serde_json::Value>>,
    // Cached users
    users: Arc<DashMap<Snowflake, serde_json::Value>>,
    // Cached channels
    channels: Arc<DashMap<Snowflake, serde_json::Value>>,
    // Self user ID
    user_id: Arc<tokio::sync::RwLock<Option<Snowflake>>>,
}

impl State {
    pub fn new() -> Self {
        State {
            guilds: Arc::new(DashMap::new()),
            users: Arc::new(DashMap::new()),
            channels: Arc::new(DashMap::new()),
            user_id: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    pub fn add_guild(&self, id: Snowflake, data: serde_json::Value) {
        self.guilds.insert(id, data);
    }

    pub fn get_guild(&self, id: Snowflake) -> Option<serde_json::Value> {
        self.guilds.get(&id).map(|v| v.clone())
    }

    pub fn remove_guild(&self, id: Snowflake) {
        self.guilds.remove(&id);
    }

    pub fn add_user(&self, id: Snowflake, data: serde_json::Value) {
        self.users.insert(id, data);
    }

    pub fn get_user(&self, id: Snowflake) -> Option<serde_json::Value> {
        self.users.get(&id).map(|v| v.clone())
    }

    pub fn add_channel(&self, id: Snowflake, data: serde_json::Value) {
        self.channels.insert(id, data);
    }

    pub fn get_channel(&self, id: Snowflake) -> Option<serde_json::Value> {
        self.channels.get(&id).map(|v| v.clone())
    }

    pub async fn set_user_id(&self, id: Snowflake) {
        *self.user_id.write().await = Some(id);
    }

    pub async fn get_user_id(&self) -> Option<Snowflake> {
        *self.user_id.read().await
    }

    pub fn clear(&self) {
        self.guilds.clear();
        self.users.clear();
        self.channels.clear();
    }
}

pub fn register_module(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
