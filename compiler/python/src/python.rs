use pyo3::{pyfunction, pymodule, types::PyModule, wrap_pyfunction, PyResult};

use slang_core::parser::parser_constructs::{ParserStatement, ParserAssemble};

use crate::bindings::{PyAttribute, PyEntrypoint, PyRule, PySlangFile};

#[pyfunction]
fn parse(s: &str) -> PyResult<PySlangFile> {
    match ParserStatement::parse(s) {
        Ok(statements) => {
            let as_py = statements.assemble();
            Ok(PySlangFile::create(as_py))
        }
        Err(e) => {
            println!("Error: {:?}", e);
            Err(pyo3::exceptions::PyException::new_err("Error parsing"))
        }
    }
}

#[pymodule]
fn slang_dsl(_py: pyo3::Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse, m)?)?;

    m.add_class::<PyEntrypoint>()?;
    m.add_class::<PyRule>()?;
    m.add_class::<PyAttribute>()?;

    Ok(())
}
