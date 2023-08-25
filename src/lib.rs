use std::fmt::{Display, Formatter};

pub mod example;

pub struct Model {}

pub struct Var {
    name: String,
}

impl Model {
    pub fn new() -> Model { Model {} }
}

impl Var {
    pub fn new(name: String) -> Var { Var { name } }
}

impl Default for Model {
    fn default() -> Self { Model::new() }
}

impl Display for Var {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}