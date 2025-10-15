/// Python bindings via PyO3
use pyo3::prelude::*;

// TODO: Implement Python bindings
// pub struct PyGraphStore { ... }

/// Initialize the Python module
#[pymodule]
fn sutra_storage(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
