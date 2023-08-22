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
