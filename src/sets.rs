use std::hash::Hash;

use fxhash::FxHashSet;
use ordered_float::NotNan;

#[derive(Clone, Debug)]
pub struct OneItemSet<T> {
    pub item: T,
    pub support_count: usize,
    pub support: NotNan<f64>,
    pub row_indexes: FxHashSet<usize>,
}

impl<T> OneItemSet<T> {
    pub(crate) const fn new(
        item: T,
        support_count: usize,
        support: NotNan<f64>,
        row_indexes: FxHashSet<usize>,
    ) -> Self
    where
        T: Clone + Eq + Hash + PartialEq,
    {
        Self { item, support_count, support, row_indexes }
    }
}

impl<T> PartialEq for OneItemSet<T>
where
    T: Eq + Hash + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item
    }
}

impl<T> Eq for OneItemSet<T> where T: Eq + Hash + PartialEq {}

#[derive(Clone, Debug)]
pub struct KItemSet<T> {
    pub items: FxHashSet<T>,
    pub support_count: usize,
    pub support: NotNan<f64>,
    pub row_indexes: FxHashSet<usize>,
}

impl<T> PartialEq for KItemSet<T>
where
    T: Eq + Hash + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}

impl<T> Eq for KItemSet<T> where T: Eq + Hash + PartialEq {}

impl<T> KItemSet<T> {
    pub(crate) fn new(
        items: FxHashSet<T>,
        data: &[FxHashSet<T>],
        row_indexes: FxHashSet<usize>,
        min_count: usize,
    ) -> Option<Self>
    where
        T: Clone + Eq + Hash + PartialEq,
    {
        if row_indexes.len() < min_count {
            return None;
        }

        let mut support_count = 0;
        let mut found_indexes = FxHashSet::default();

        for index in row_indexes {
            if data[index].is_superset(&items) {
                support_count += 1;
                found_indexes.insert(index);
            }
        }

        if support_count < min_count {
            return None;
        }

        let support = NotNan::new((support_count as f64) / (data.len() as f64))
            .expect("never NaN");

        Some(Self { items, support_count, support, row_indexes: found_indexes })
    }
}
