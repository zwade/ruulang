use pyo3::{pyfunction, pymodule, types::PyModule, wrap_pyfunction, PyResult};

use crate::{
    parser::slang_ast::{Attribute, Entrypoint, Rule},
    slang::TermParser,
};

#[pyfunction]
fn parse(s: &str) -> PyResult<Vec<Entrypoint>> {
    let as_py = TermParser::new().parse(s).unwrap();
    Ok(as_py)
}

#[pyfunction]
fn json_dump(entrypoints: Vec<Entrypoint>) -> PyResult<String> {
    let as_json = serde_json::to_string(&entrypoints).unwrap();
    Ok(as_json)
}

#[pymodule]
fn slang_dsl(_py: pyo3::Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(json_dump, m)?)?;

    m.add_class::<Entrypoint>()?;
    m.add_class::<Rule>()?;
    m.add_class::<Attribute>()?;

    Ok(())
}
