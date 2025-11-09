use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

/// Discord Gateway Intents
///
/// Represents which events your bot will receive from Discord's Gateway
#[pyclass]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Intents {
    #[pyo3(get, set)]
    pub value: i64,
}

#[pymethods]
impl Intents {
    #[new]
    #[pyo3(signature = (value=0, **kwargs))]
    fn new(value: i64, kwargs: Option<&Bound<'_, pyo3::types::PyDict>>) -> PyResult<Self> {
        let mut intents = Intents { value };

        if let Some(kw) = kwargs {
            for (key, val) in kw.iter() {
                let key_str: String = key.extract()?;
                let bool_val: bool = val.extract()?;

                let flag = match key_str.as_str() {
                    "guilds" => 1 << 0,
                    "members" => 1 << 1,
                    "moderation" | "bans" => 1 << 2,
                    "emojis" | "emojis_and_stickers" | "expressions" => 1 << 3,
                    "integrations" => 1 << 4,
                    "webhooks" => 1 << 5,
                    "invites" => 1 << 6,
                    "voice_states" => 1 << 7,
                    "presences" => 1 << 8,
                    "messages" | "guild_messages" => 1 << 9,
                    "dm_messages" => 1 << 12,
                    "reactions" | "guild_reactions" => 1 << 10,
                    "dm_reactions" => 1 << 13,
                    "typing" | "guild_typing" => 1 << 11,
                    "dm_typing" => 1 << 14,
                    "message_content" => 1 << 15,
                    "guild_scheduled_events" => 1 << 16,
                    "auto_moderation" | "auto_moderation_configuration" => 1 << 20,
                    "auto_moderation_execution" => 1 << 21,
                    "polls" | "guild_polls" => 1 << 24,
                    "dm_polls" => 1 << 25,
                    _ => return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                        format!("'{}' is not a valid flag name.", key_str)
                    )),
                };

                if bool_val {
                    intents.value |= flag;
                } else {
                    intents.value &= !flag;
                }
            }
        }

        Ok(intents)
    }

    #[staticmethod]
    fn all() -> Self {
        Intents {
            value: (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3) | (1 << 4) |
                   (1 << 5) | (1 << 6) | (1 << 7) | (1 << 8) | (1 << 9) |
                   (1 << 10) | (1 << 11) | (1 << 12) | (1 << 13) | (1 << 14) |
                   (1 << 15) | (1 << 16) | (1 << 20) | (1 << 21) | (1 << 24) | (1 << 25),
        }
    }

    #[staticmethod]
    fn none() -> Self {
        Intents { value: 0 }
    }

    #[staticmethod]
    pub fn default() -> Self {
        let mut intents = Self::all();
        intents.value &= !(1 << 8); // presences
        intents.value &= !(1 << 1); // members
        intents.value &= !(1 << 15); // message_content
        intents
    }

    fn __or__(&self, other: &Self) -> Self {
        Intents { value: self.value | other.value }
    }

    fn __and__(&self, other: &Self) -> Self {
        Intents { value: self.value & other.value }
    }

    fn __xor__(&self, other: &Self) -> Self {
        Intents { value: self.value ^ other.value }
    }

    fn __invert__(&self) -> Self {
        Intents { value: !self.value }
    }

    fn __repr__(&self) -> String {
        format!("<Intents value={}>", self.value)
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.value == other.value
    }

    fn __bool__(&self) -> bool {
        self.value != 0
    }

    #[getter]
    fn guilds(&self) -> bool {
        (self.value & (1 << 0)) != 0
    }

    #[setter]
    fn set_guilds(&mut self, value: bool) {
        if value {
            self.value |= 1 << 0;
        } else {
            self.value &= !(1 << 0);
        }
    }

    #[getter]
    fn members(&self) -> bool {
        (self.value & (1 << 1)) != 0
    }

    #[setter]
    fn set_members(&mut self, value: bool) {
        if value {
            self.value |= 1 << 1;
        } else {
            self.value &= !(1 << 1);
        }
    }

    #[getter]
    fn message_content(&self) -> bool {
        (self.value & (1 << 15)) != 0
    }

    #[setter]
    fn set_message_content(&mut self, value: bool) {
        if value {
            self.value |= 1 << 15;
        } else {
            self.value &= !(1 << 15);
        }
    }
}

/// Discord presence status
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Online,
    Offline,
    Idle,
    #[serde(rename = "dnd")]
    Dnd,
    Invisible,
}

pub fn register_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Intents>()?;
    Ok(())
}
