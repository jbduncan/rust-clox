use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Value {
    Bool(bool),
    Nil,
    Number(f64),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(boolean) => write!(f, "{}", boolean),
            Value::Nil => write!(f, "nil"),
            Value::Number(number) => write!(f, "{}", number),
        }
    }
}
