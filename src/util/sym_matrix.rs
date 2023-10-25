use std::ops::{Index, IndexMut};

pub(crate) struct SymMatrix<T: Clone> {
    pub(crate) size: usize,
    pub(crate) elements: Vec<T>,
}

impl<T: Clone> SymMatrix<T> {
    pub(crate) fn new(element: T, size: usize) -> SymMatrix<T> {
        let n_elements = size * (size + 1) / 2;  //  Triangular number formula
        let elements: Vec<T> = vec![element; n_elements];
        SymMatrix { size, elements }
    }
    fn i_element(i1: usize, i2: usize) -> usize {
        let (i_min, i_max) = if i1 <= i2 { (i1, i2) } else { (i2, i1) };
        i_max * (i_max + 1) / 2 + i_min
    }
}

impl<T: Clone> Index<(usize, usize)> for SymMatrix<T> {
    type Output = T;

    fn index(&self, (i1, i2): (usize, usize)) -> &Self::Output {
        &self.elements[SymMatrix::<T>::i_element(i1, i2)]
    }
}

impl<T: Clone> IndexMut<(usize, usize)> for SymMatrix<T> {
    fn index_mut(&mut self, (i1, i2): (usize, usize)) -> &mut Self::Output {
        &mut self.elements[SymMatrix::<T>::i_element(i1, i2)]
    }
}
