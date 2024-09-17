use ducky_exec::ast::{DuckyScript, ParserEvents};
use pyo3::{exceptions::PyValueError, prelude::*};
use std::{
    sync::{
        mpsc::{channel, Receiver},
        Arc,
    },
    thread::spawn,
};

#[pyclass(module = "ducky_interpreter")]
struct DuckyExec {
    interpreter: DuckyScript,
    // done: Arc<Mutex<bool>>,
    // tx: Sender<ParserEvents>,
    rx: Receiver<ParserEvents>,
    total_lines: usize,
}

#[pymethods]
impl DuckyExec {
    #[new]
    fn new(serial_dev_path: &str) -> Self {
        let (tx, rx) = channel();

        Self {
            interpreter: DuckyScript::new(serial_dev_path, tx),
            // done: Arc::new(Mutex::new(false)),
            rx,
            total_lines: 0,
        }
    }

    fn set_dev(&mut self, dev_path: &str) {
        let (tx, rx) = channel();

        self.rx = rx;
        self.interpreter = DuckyScript::new(dev_path, tx);
    }

    fn exec_script(&mut self, code: &str) -> PyResult<()> {
        let interpreter = self.interpreter.clone();
        let code: Arc<str> = code.into();
        self.total_lines = code.chars().filter(|c| *c == '\n').count();

        spawn(move || {
            if let Err(e) = interpreter.from_source(&code) {
                // println!("{e}");
                Err(PyErr::new::<PyValueError, String>(e.to_string()))
            } else {
                Ok(())
            }
        });

        Ok(())
    }

    fn step(&mut self) -> PyResult<Option<usize>> {
        match self.rx.recv().unwrap() {
            ParserEvents::Done(Ok(_)) => Ok(None),
            ParserEvents::Done(Err(e)) => Err(PyErr::new::<PyValueError, String>(e.to_string())),
            ParserEvents::Line(line_n) => Ok(Some(line_n)),
        }
    }

    fn get_total_lines(&self) -> usize {
        self.total_lines
    }

    // fn done(&self) -> PyResult<bool> {
    //     Ok(*self.done.lock().unwrap())
    // }

    // fn get_line(&self) -> PyResult<usize> {
    //     Ok(*self.interpreter.line.lock().unwrap())
    // }
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
