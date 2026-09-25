use std::{collections::{HashSet}, hash::Hash};

use ordered_float::NotNan;

#[derive(Clone, Debug)]
pub struct OneItemSet<T> {
    pub item: T,
    pub support_count: usize,
    pub support: NotNan<f64>,
    pub row_indexes: HashSet<usize>,
}

impl<T> OneItemSet<T> {
    pub(crate) const fn new(
        item: T,
        support_count: usize,
        support: NotNan<f64>,
        row_indexes: HashSet<usize>,
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
    pub items: HashSet<T>,
    pub support_count: usize,
    pub support: NotNan<f64>,
    pub row_indexes: HashSet<usize>,
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
    pub(crate) fn new<I>(
        items: HashSet<T>,
        data: &[HashSet<T>],
        row_indexes: I,
    ) -> Self
    where
        T: Clone + Eq + Hash + PartialEq,
        I: IntoIterator<Item = usize>,
    {
        let mut support_count = 0;
        let mut found_indexes = HashSet::new();

        for index in row_indexes {
            if data[index].is_superset(&items) {
                support_count += 1;
                found_indexes.insert(index);
            }
        }

        let support = NotNan::new((support_count as f64) / (data.len() as f64))
            .expect("never NaN");

        Self { items, support_count, support, row_indexes: found_indexes }
    }
}
