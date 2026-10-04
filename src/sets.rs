use std::{
    collections::BTreeSet,
    fmt::{self, Display},
    iter,
};

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
    ) -> Self {
        Self { item, support_count, support, row_indexes }
    }
}

impl<T> PartialEq for OneItemSet<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item
    }
}

impl<T> Eq for OneItemSet<T> where T: Eq {}

#[derive(Clone, Debug)]
pub struct KItemSet<T> {
    pub items: BTreeSet<T>,
    pub support_count: usize,
    pub support: NotNan<f64>,
    pub row_indexes: FxHashSet<usize>,
}

impl<T> PartialEq for KItemSet<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}

impl<T> Eq for KItemSet<T> where T: Eq {}

impl<T> KItemSet<T> {
    pub(crate) fn new(
        items: BTreeSet<T>,
        data: &[BTreeSet<T>],
        row_indexes: FxHashSet<usize>,
        min_support: f64,
    ) -> Option<Self>
    where
        T: Ord,
    {
        if row_indexes.len() < (min_support * data.len() as f64) as usize {
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

        let support = NotNan::new((support_count as f64) / (data.len() as f64))
            .expect("never NaN");

        if support.into_inner() < min_support {
            return None;
        }

        Some(Self { items, support_count, support, row_indexes: found_indexes })
    }
}

#[derive(Clone, Debug)]
pub enum AnySet<T> {
    OneItem(OneItemSet<T>),
    KItem(KItemSet<T>),
}

impl<T> AnySet<T> {
    pub const fn support(&self) -> NotNan<f64> {
        match self {
            Self::OneItem(one_item_set) => one_item_set.support,
            Self::KItem(k_item_set) => k_item_set.support,
        }
    }

    pub fn items(&self) -> BTreeSet<T>
    where
        T: Clone + Ord,
    {
        match self {
            Self::OneItem(one_item_set) => {
                iter::once(one_item_set.item.clone()).collect()
            }
            Self::KItem(k_item_set) => k_item_set.items.clone(),
        }
    }

    pub fn indices(&self) -> &FxHashSet<usize> {
        match self {
            Self::OneItem(one_item_set) => &one_item_set.row_indexes,
            Self::KItem(k_item_set) => &k_item_set.row_indexes,
        }
    }
}

impl<T> Display for AnySet<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ ")?;

        match self {
            Self::OneItem(one_item_set) => {
                write!(f, "{}", one_item_set.item)?;
            }
            Self::KItem(k_item_set) => {
                for (i, item) in k_item_set.items.iter().enumerate() {
                    write!(f, "{item}")?;

                    if i != k_item_set.items.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
            }
        }

        write!(f, " }} support = {:.2}", self.support())?;

        Ok(())
    }
}
