use ducky_exec::ast::DuckyScript;
use pyo3::prelude::*;

#[pyclass(module = "ducky_interpreter")]
struct DuckyExec {
    interpreter: DuckyScript,
}

#[pymethods]
impl DuckyExec {
    #[new]
    fn new(serial_dev_path: &str) -> Self {
        Self {
            interpreter: DuckyScript::new(serial_dev_path),
        }
    }

    fn set_dev(&mut self, dev_path: &str) {
        self.interpreter = DuckyScript::new(dev_path);
    }

    fn exec_script(&mut self, code: &str) -> PyResult<()> {
        if let Err(e) = self.interpreter.from_source(code) {
            Err(PyErr::new::<PyAny, String>(e.to_string()))
        } else {
            Ok(())
        }
    }

    fn get_line(&self) -> PyResult<usize> {
        Ok(self.interpreter.line)
    }
}

// /// Formats the sum of two numbers as string.
// #[pyfunction]
// fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
//     Ok((a + b).to_string())
// }

/// A Python module implemented in Rust.
#[pymodule]
fn ducky_interpreter(_py: Python, m: &PyModule) -> PyResult<()> {
    // m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_class::<DuckyExec>()?;
    Ok(())
}
