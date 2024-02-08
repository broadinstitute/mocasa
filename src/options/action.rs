
pub(crate) mod names {
    pub(crate) const TRAIN: &str = "train";
    pub(crate) const CLASSIFY: &str = "classify";
}


#[derive(PartialEq, Clone, Copy)]
pub enum Action { Train, Classify }
