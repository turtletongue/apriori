use std::{
    collections::BTreeSet,
    fmt::{self, Display},
};

use ordered_float::NotNan;

#[derive(Clone, Debug)]
pub struct AssociationRule<T> {
    pub antecedent: BTreeSet<T>,
    pub consequent: BTreeSet<T>,
    pub confidence: NotNan<f64>,
}

impl<T> AssociationRule<T> {
    pub(crate) const fn new(
        antecedent: BTreeSet<T>,
        consequent: BTreeSet<T>,
        confidence: NotNan<f64>,
    ) -> Self {
        Self { antecedent, consequent, confidence }
    }
}

impl<T> Display for AssociationRule<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ ")?;

        for (i, item) in self.antecedent.iter().enumerate() {
            write!(f, "{item}")?;

            if i != self.antecedent.len() - 1 {
                write!(f, ", ")?;
            }
        }

        write!(f, " }} -> {{ ")?;

        for (i, item) in self.consequent.iter().enumerate() {
            write!(f, "{item}")?;

            if i != self.consequent.len() - 1 {
                write!(f, ", ")?;
            }
        }

        write!(f, " }} confidence = {:.2}", self.confidence)?;

        Ok(())
    }
}
