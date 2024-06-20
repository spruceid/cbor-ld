use std::borrow::Borrow;

#[derive(Debug)]
pub struct Spaceless<T>(pub T);

impl<T: Borrow<str>, U: Borrow<str>> PartialEq<Spaceless<U>> for Spaceless<T> {
    fn eq(&self, other: &Spaceless<U>) -> bool {
        let t: &str = self.0.borrow();
        let u: &str = other.0.borrow();

        t.chars()
            .filter(|c| !c.is_whitespace())
            .eq(u.chars().filter(|c| !c.is_whitespace()))
    }
}
