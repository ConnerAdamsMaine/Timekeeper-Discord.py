use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Discord Snowflake ID type
/// Represented as u64 internally, but can accept str or int from Python
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Snowflake(pub u64);

impl Snowflake {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Extract the timestamp from a snowflake ID
    pub fn timestamp(&self) -> i64 {
        ((self.0 >> 22) + 1420070400000) as i64
    }

    /// Get the worker ID from the snowflake
    pub fn worker_id(&self) -> u8 {
        ((self.0 & 0x3E0000) >> 17) as u8
    }

    /// Get the process ID from the snowflake
    pub fn process_id(&self) -> u8 {
        ((self.0 & 0x1F000) >> 12) as u8
    }

    /// Get the increment from the snowflake
    pub fn increment(&self) -> u16 {
        (self.0 & 0xFFF) as u16
    }
}

impl fmt::Display for Snowflake {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for Snowflake {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<Snowflake> for u64 {
    fn from(snowflake: Snowflake) -> Self {
        snowflake.0
    }
}

impl<'py> FromPyObject<'py> for Snowflake {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        // Try to extract as int first
        if let Ok(id) = ob.extract::<u64>() {
            return Ok(Snowflake(id));
        }
        // Try to extract as string and parse
        if let Ok(s) = ob.extract::<String>() {
            match s.parse::<u64>() {
                Ok(id) => Ok(Snowflake(id)),
                Err(_) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("Invalid snowflake string: {}", s)
                )),
            }
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Snowflake must be int or str"
            ))
        }
    }
}

impl IntoPy<PyObject> for Snowflake {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.0.into_py(py)
    }
}

pub fn register_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Snowflakes are just exposed as ints to Python for compatibility
    Ok(())
}
