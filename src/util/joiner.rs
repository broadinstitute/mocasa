use std::fmt::Display;

pub(crate) struct Joiner<'a, 'b, T: Display> {
    sep: &'a str,
    list: &'b[T]
}

impl <'a, 'b, T: Display> Joiner<'a,'b, T> {
    pub(crate) fn new(sep: &'a str, list: &'b[T]) -> Joiner<'a, 'b, T> {
        Joiner { sep, list }
    }
}

impl<'a, 'b, T: Display> Display for Joiner<'a, 'b, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut iter = self.list.iter();
        if let Some(first) = iter.next() {
            write!(f, "{}", first)?;
            for item in iter {
                write!(f, "{}{}", self.sep, item)?;
            }
        }
        Ok(())
    }
}