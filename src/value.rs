pub struct Value(pub f32);

impl Value {
    pub fn print(&self) {
        print!("{}", self.0);
    }
}
