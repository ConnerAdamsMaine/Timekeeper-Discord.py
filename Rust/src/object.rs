use pyo3::prelude::*;
use crate::_types::Snowflake;

const DISCORD_EPOCH: i64 = 1420070400000;

/// Represents a generic Discord object.
///
/// The purpose of this class is to allow you to create 'miniature'
/// versions of data classes if you want to pass in just an ID.
#[pyclass]
#[derive(Clone)]
pub struct Object {
    #[pyo3(get, set)]
    pub id: u64,
}

#[pymethods]
impl Object {
    #[new]
    #[pyo3(signature = (id))]
    fn new(id: PyObject, py: Python) -> PyResult<Self> {
        // Try to convert to u64
        let id_val = if let Ok(i) = id.extract::<u64>(py) {
            i
        } else if let Ok(s) = id.extract::<String>(py) {
            s.parse::<u64>().map_err(|_| {
                PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    format!("id parameter must be convertible to int not {}", s)
                )
            })?
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "id parameter must be convertible to int"
            ));
        };

        Ok(Object { id: id_val })
    }

    fn __repr__(&self) -> String {
        format!("<Object id={}>", self.id)
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.id == other.id
    }

    fn __hash__(&self) -> u64 {
        self.id
    }

    #[getter]
    fn created_at(&self, py: Python) -> PyResult<PyObject> {
        let timestamp_ms = (self.id >> 22) + DISCORD_EPOCH as u64;
        let timestamp_secs = (timestamp_ms / 1000) as f64;

        // Create Python datetime
        let datetime = py.import_bound("datetime")?;
        let dt = datetime.call_method1("fromtimestamp", (timestamp_secs,))?;

        Ok(dt.into())
    }
}

/// Oldest possible Discord object
#[pyfunction]
fn oldest_object() -> Object {
    Object { id: 0 }
}

pub fn register_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Object>()?;
    m.add_function(wrap_pyfunction!(oldest_object, m)?)?;
    Ok(())
}
