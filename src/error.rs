use std::fmt;
use std::error::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum SchemeError {
    SyntaxError(String),
    TypeError(String),
    UndefinedSymbol(String),
    ArityError(String),
    DivisionByZero,
    RuntimeError(String),
}

impl fmt::Display for SchemeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SchemeError::SyntaxError(msg) => write!(f, "syntax error: {}", msg),
            SchemeError::TypeError(msg) => write!(f, "type error: {}", msg),
            SchemeError::UndefinedSymbol(msg) => write!(f, "undefined symbol: {}", msg),
            SchemeError::ArityError(msg) => write!(f, "arity error: {}", msg),
            SchemeError::DivisionByZero => write!(f, "division by zero"),
            SchemeError::RuntimeError(msg) => write!(f, "runtime error: {}", msg),
        }
    }
}

impl Error for SchemeError {}

impl From<&'static str> for SchemeError {
    fn from(s: &'static str) -> Self {
        if s.contains("not defined") {
            SchemeError::UndefinedSymbol(s.to_string())
        } else if s.contains("syntax") || s.contains("unexpected") || s.contains("end quote")
            || s.contains("number of parts") || s.contains("lambda argument")
            || s.contains("unsupported data type")
        {
            SchemeError::SyntaxError(s.to_string())
        } else if s.contains("wrong argument datatype") || s.contains("wrong type")
            || s.contains("of type '") || s.contains("requires an argument of type")
        {
            SchemeError::TypeError(s.to_string())
        } else if s.contains("requires") && s.contains("argument") {
            SchemeError::ArityError(s.to_string())
        } else {
            SchemeError::RuntimeError(s.to_string())
        }
    }
}
