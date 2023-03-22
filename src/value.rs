use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq)]
// TODO: Is it possible to allow Value::Strings to
//       share the same String with Rc? If so, add
//       Copy again.
#[repr(C)] // TODO: Is this needed?
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
    String(String),
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Value::Nil => true,
            Value::Bool(value) => !value,
            _ => false,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(boolean) => write!(f, "{boolean}"),
            Value::Nil => write!(f, "nil"),
            Value::Number(number) => write!(f, "{number}"),
            Value::String(string) => write!(f, "{string}"),
        }
    }
}
