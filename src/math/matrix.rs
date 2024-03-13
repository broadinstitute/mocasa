use std::fmt::Formatter;
use std::ops::{Index, IndexMut};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{SeqAccess, Visitor, Error as SerdeError};
use serde::ser::SerializeSeq;
use crate::error::Error;

#[derive(Clone)]
pub(crate) struct Matrix {
    pub(crate) n_rows: usize,
    pub(crate) n_cols: usize,
    pub(crate) elements: Vec<f64>,
}

impl Matrix {
    pub(crate) fn fill<F: FnMut(usize, usize) -> f64>(n_rows: usize, n_cols: usize, mut f: F)
                                                      -> Matrix {
        let n = n_rows * n_cols;
        let elements = {
            let mut elements: Vec<f64> = Vec::with_capacity(n);
            for i_row in 0..n_rows {
                for i_col in 0..n_cols {
                    elements.push(f(i_row, i_col))
                }
            }
            elements
        };
        Matrix { n_rows, n_cols, elements }
    }
    pub(crate) fn from_vec(n_rows: usize, n_cols: usize, elements: Vec<f64>)
                           -> Result<Matrix, Error> {
        if elements.len() == n_rows * n_cols {
            Ok(Matrix { n_rows, n_cols, elements })
        } else {
            Err(Error::from(
                format!("With {} rows and {} columns, expecting {} elements, but got {}",
                        n_rows, n_cols, n_rows * n_cols, elements.len())
            ))
        }
    }
    pub(crate) fn only_cols(&self, is_col: &[usize]) -> Matrix {
        let n_rows = self.n_rows;
        let n_cols = is_col.len();
        let mut selection = Matrix::fill(n_rows, n_cols, |_,_| 0.0);
        for i_row in 0..n_rows {
            for (i_col_new, &i_col_old) in is_col.iter().enumerate() {
                selection[i_row][i_col_new] = self[i_row][i_col_old];
            }
        }
        selection
    }
}

impl Index<usize> for Matrix {
    type Output = [f64];

    fn index(&self, i_row: usize) -> &Self::Output {
        let from = i_row * self.n_cols;
        let to = from + self.n_cols;
        &self.elements[from..to]
    }
}

impl IndexMut<usize> for Matrix {
    fn index_mut(&mut self, i_row: usize) -> &mut Self::Output {
        let from = i_row * self.n_cols;
        let to = from + self.n_cols;
        &mut self.elements[from..to]
    }
}

impl Serialize for Matrix {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut seq = serializer.serialize_seq(Some(self.n_rows))?;
        for i_row in 0..self.n_rows {
            seq.serialize_element(&self[i_row])?
        }
        seq.end()
    }
}

struct MatrixDesVisitor {}

impl MatrixDesVisitor {
    fn new() -> MatrixDesVisitor {
        MatrixDesVisitor {}
    }
}

impl<'de> Visitor<'de> for MatrixDesVisitor {
    type Value = Matrix;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a matrix (array of arrays of numbers)")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
        let mut n_rows: usize = 0;
        let mut n_cols: usize = 0;
        let mut elements: Vec<f64> = Vec::new();
        if let Some(mut row0) = seq.next_element::<Vec<f64>>()? {
            n_rows = 1;
            n_cols = row0.len();
            elements.append(&mut row0);
            while let Some(mut row) = seq.next_element::<Vec<f64>>()? {
                if row.len() != n_cols {
                    return Err(A::Error::invalid_length(row.len(), &self));
                }
                elements.append(&mut row);
                n_rows += 1;
            }
        }
        Ok(Matrix { n_rows, n_cols, elements })
    }
}

impl<'de> Deserialize<'de> for Matrix {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_seq(MatrixDesVisitor::new())
    }
}