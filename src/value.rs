use std::fmt::{Display, Formatter};

pub struct Value(pub f32);

impl Value {
    pub fn print(&self) {
        print!("{}", self.0);
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
