use pyo3::{pyfunction, pymodule, types::PyModule, wrap_pyfunction, PyResult};

use slang_core::slang::TermParser;

use crate::bindings::{PyAttribute, PyEntrypoint, PyRule, PySlangFile};

#[pyfunction]
fn parse(s: &str) -> PyResult<PySlangFile> {
    let as_py = TermParser::new().parse(s).unwrap();
    Ok(PySlangFile::create(as_py))
}

#[pymodule]
fn slang_dsl(_py: pyo3::Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse, m)?)?;

    m.add_class::<PyEntrypoint>()?;
    m.add_class::<PyRule>()?;
    m.add_class::<PyAttribute>()?;

    Ok(())
}
