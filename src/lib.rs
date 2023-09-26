use std::collections::HashMap;
use pyo3::{prelude::*};
use cel_interpreter::{Context, Program, Value};

/* As I understood `unsandable` prevents class from being thread-safe and
   there will be error when accessed from a differen thread:

thread '<unnamed>' panicked at 'assertion failed: `(left == right)`
  left: `ThreadId(2)`,
 right: `ThreadId(1)`: rust_python_example::MyProgram is unsendable, but sent to another thread!', /Users/inikolaev/.cargo/registry/src/index.crates.io-6f17d22bba15001f/pyo3-0.19.2/src/impl_/pyclass.rs:927:9
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
Exception in thread Thread-1 (worker):
Traceback (most recent call last):
  File "/Library/Frameworks/Python.framework/Versions/3.11/lib/python3.11/threading.py", line 1038, in _bootstrap_inner
>>>     self.run()
  File "/Library/Frameworks/Python.framework/Versions/3.11/lib/python3.11/threading.py", line 975, in run
    self._target(*self._args, **self._kwargs)
  File "<stdin>", line 2, in worker
pyo3_runtime.PanicException: assertion failed: `(left == right)`
  left: `ThreadId(2)`,
 right: `ThreadId(1)`: rust_python_example::MyProgram is unsendable, but sent to another thread!    
*/
#[pyclass]
struct MyProgram {
    program: Program
}

#[derive(FromPyObject)]
enum CelValue {
    CelBool {
        #[pyo3(attribute("value"))]
        value: bool,
    },
    CelInt {
        #[pyo3(attribute("value"))]
        value: i32,
    },
    CelString {
        #[pyo3(attribute("value"))]
        value: String,
    },
    CelFloat{
        #[pyo3(attribute("value"))]
        value: f64,
    },
}

pub trait ToCelValue {
    fn to_cel_value(&self) -> Value;
}

impl ToCelValue for CelValue {
    fn to_cel_value(&self) -> Value {
        match self {
            CelValue::CelBool {value} => {
                Value::Bool(*value)
            },
            CelValue::CelInt {value} => {
                Value::Int(*value)
            },
            CelValue::CelFloat {value} => {
                Value::Float(*value)
            },
            CelValue::CelString {value} => {
                Value::String((*value).clone().into())
            },
        }
    }
}

#[pymethods]
impl MyProgram {
    #[new]
    fn new(expr: String) -> Self {
        let program = Program::compile(&expr).unwrap();
        MyProgram { program }
    }

    fn evaluate(&mut self, ctx: HashMap<String, CelValue>) -> PyResult<bool> {
        let mut context = Context::default();

        for (name, value) in ctx {
            context.add_variable(name, value.to_cel_value());
        }

        let result = self.program.execute(&context);

        match result {
            Ok(Value::Bool(value)) => Ok(value),
            _ => Ok(false)
        }
    }
}

// Implement Send to tell compiler that it's thread-safe?
unsafe impl Send for MyProgram {}

/// A Python module implemented in Rust.
#[pymodule]
fn rust_python_example(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<MyProgram>()?;
    Ok(())
}