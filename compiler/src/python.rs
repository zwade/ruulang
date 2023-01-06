use pyo3::{PyResult, pyfunction, pymodule, types::PyModule, wrap_pyfunction};

use crate::{slang::TermParser, parser::slang_ast::{Entrypoint, Rule, Attribute}};

#[pyfunction]
fn parse(s: &str) -> PyResult<Vec<Entrypoint>> {
    let as_py = TermParser::new().parse(s).unwrap();
    Ok(as_py)
}

#[pymodule]
fn slang_dsl(_py: pyo3::Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse, m)?)?;

    m.add_class::<Entrypoint>()?;
    m.add_class::<Rule>()?;
    m.add_class::<Attribute>()?;

    Ok(())
}