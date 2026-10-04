use std::{
    collections::{BTreeSet, HashMap, HashSet},
    hash::Hash,
    iter,
};

use fxhash::{FxHashMap, FxHashSet};
use itertools::Itertools as _;
use ordered_float::NotNan;

pub use crate::{
    rules::AssociationRule,
    sets::{AnySet, KItemSet, OneItemSet},
};

mod rules;
mod sets;

#[must_use]
pub fn generate_one_item_sets<'a, T>(
    data: &[BTreeSet<&'a T>],
    min_support: f64,
) -> Vec<OneItemSet<&'a T>>
where
    T: Hash + Ord + ?Sized,
{
    let map = data.iter().enumerate().fold(
        FxHashMap::default(),
        |mut map, (i, transaction)| {
            for &item in transaction {
                map.entry(item)
                    .and_modify(|(count, indexes): &mut (_, FxHashSet<_>)| {
                        *count += 1;
                        indexes.insert(i);
                    })
                    .or_insert_with(|| (1, iter::once(i).collect()));
            }

            map
        },
    );

    map.into_iter()
        .filter_map(|(item, (count, indexes))| {
            let support = NotNan::new((count as f64) / (data.len() as f64))
                .expect("never NaN");

            if support.into_inner() < min_support {
                return None;
            }

            Some((item, support, (count, indexes)))
        })
        .map(|(item, support, (count, indexes))| {
            OneItemSet::new(item, count, support, indexes)
        })
        .collect()
}

#[must_use]
pub fn generate_two_item_sets<'a, T>(
    data: &[BTreeSet<&'a T>],
    one_item_sets: &[OneItemSet<&'a T>],
    min_support: f64,
) -> Vec<KItemSet<&'a T>>
where
    T: Ord + ?Sized,
{
    one_item_sets
        .iter()
        .enumerate()
        .flat_map(|(i, a)| {
            one_item_sets.iter().skip(i + 1).filter_map(|b| {
                let merged: BTreeSet<_> =
                    [a.item, b.item].into_iter().collect();

                let row_indexes: FxHashSet<_> = a
                    .row_indexes
                    .intersection(&b.row_indexes)
                    .copied()
                    .collect();

                KItemSet::new(merged, data, row_indexes, min_support)
            })
        })
        .collect()
}

#[must_use]
pub fn generate_next_sets<'a, T>(
    data: &[BTreeSet<&'a T>],
    previous_sets: &[KItemSet<&'a T>],
    min_support: f64,
) -> Vec<KItemSet<&'a T>>
where
    T: Hash + Ord + ?Sized,
{
    let subsets: HashSet<BTreeSet<_>> = previous_sets
        .into_iter()
        .map(|set| set.items.iter().copied().collect())
        .collect();

    previous_sets
        .iter()
        .enumerate()
        .flat_map(|(i, a)| {
            let mut sets = vec![];

            for b in previous_sets.iter().skip(i + 1) {
                let Some(merged) = merge(&a.items, &b.items, &subsets) else {
                    continue;
                };

                let row_indexes: FxHashSet<_> = a
                    .row_indexes
                    .intersection(&b.row_indexes)
                    .copied()
                    .collect();

                let Some(set) =
                    KItemSet::new(merged, data, row_indexes, min_support)
                else {
                    continue;
                };

                sets.push(set);
            }

            sets
        })
        .collect()
}

#[must_use]
pub fn generate_association_rules<'a, T, I>(
    one_item_sets: &[OneItemSet<&'a T>],
    last_k_item_sets: I,
    min_confidence: f64,
) -> Vec<AssociationRule<&'a T>>
where
    I: IntoIterator,
    I::Item: IntoIterator<Item = KItemSet<&'a T>>,
    T: Hash + Ord + ?Sized,
{
    let mut rules = vec![];
    let mut support_map: HashMap<BTreeSet<&'a T>, NotNan<f64>> = one_item_sets
        .into_iter()
        .map(|set| (iter::once(set.item).collect(), set.support))
        .collect();

    for item_sets in last_k_item_sets {
        let item_sets = item_sets.into_iter();

        for set in item_sets {
            let items: BTreeSet<_> = set.items.clone().into_iter().collect();

            let mut consequents: Vec<BTreeSet<_>> = set
                .items
                .into_iter()
                .map(|item| iter::once(item).collect())
                .collect();

            while !consequents.is_empty() {
                let mut consequent_candidates = HashSet::new();

                for consequent in consequents {
                    let antecedent: BTreeSet<_> =
                        items.difference(&consequent).copied().collect();

                    let Some(antecedent_support) = support_map.get(&antecedent)
                    else {
                        continue;
                    };

                    let Some(consequent_support) = support_map.get(&consequent)
                    else {
                        continue;
                    };

                    let confidence = set.support / antecedent_support;

                    if confidence.into_inner() < min_confidence {
                        continue;
                    }

                    rules.push(AssociationRule::new(
                        antecedent,
                        consequent.clone(),
                        confidence,
                        confidence / consequent_support,
                    ));

                    consequent_candidates.insert(consequent);
                }

                consequents = consequent_candidates
                    .iter()
                    .enumerate()
                    .flat_map(|(i, a)| {
                        consequent_candidates
                            .iter()
                            .skip(i + 1)
                            .filter_map(|b| merge(a, b, &consequent_candidates))
                    })
                    .collect();
            }

            support_map.insert(items, set.support);
        }
    }

    rules
}

fn merge<'a, T>(
    a: &BTreeSet<&'a T>,
    b: &BTreeSet<&'a T>,
    subsets: &HashSet<BTreeSet<&'a T>>,
) -> Option<BTreeSet<&'a T>>
where
    T: Hash + Ord + ?Sized,
{
    let k = a.len();

    let is_common_prefix = a.iter().take(k - 1).eq(b.iter().take(k - 1));

    if !is_common_prefix {
        return None;
    }

    let merged: BTreeSet<_> =
        a.iter().copied().chain(b.iter().copied()).collect();

    for subset in merged.iter().combinations(k) {
        let subset: BTreeSet<_> = subset.into_iter().copied().collect();

        if !subsets.contains(&subset) {
            return None;
        }
    }

    Some(merged)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_data() -> Vec<BTreeSet<&'static str>> {
        vec![
            BTreeSet::from_iter(["Bread", "Butter", "Milk"]),
            BTreeSet::from_iter(["Bread", "Butter"]),
            BTreeSet::from_iter(["Bread", "Milk"]),
            BTreeSet::from_iter(["Butter", "Milk"]),
            BTreeSet::from_iter(["Bread", "Milk"]),
        ]
    }

    #[test]
    fn one_item_sets() {
        let data = get_data();
        let sets = generate_one_item_sets(&data, 0.5);

        assert_eq!(sets.len(), 3);

        let bread = sets.iter().find(|set| set.item == "Bread").unwrap();

        assert_eq!(bread.support_count, 4);
        assert_eq!(bread.support, NotNan::new(0.8).unwrap());
    }

    #[test]
    fn two_item_sets() {
        let data = get_data();
        let one_item_sets = generate_one_item_sets(&data, 0.5);

        let sets = generate_two_item_sets(&data, &one_item_sets, 0.5);

        assert_eq!(sets.len(), 1);

        let bread_milk = sets
            .iter()
            .find(|set| {
                set.items.is_superset(&["Bread", "Milk"].into_iter().collect())
            })
            .unwrap();

        assert_eq!(bread_milk.support_count, 3);
        assert_eq!(bread_milk.support, NotNan::new(0.6).unwrap());
    }

    #[test]
    fn three_item_sets() {
        let data = get_data();
        let one_item_sets = generate_one_item_sets(&data, 0.5);
        let two_item_sets = generate_two_item_sets(&data, &one_item_sets, 0.5);

        let sets = generate_next_sets(&data, &two_item_sets, 0.5);

        assert_eq!(sets.len(), 0);
    }

    #[test]
    fn association_rules() {
        let data = get_data();
        let one_item_sets = generate_one_item_sets(&data, 0.5);
        let two_item_sets = generate_two_item_sets(&data, &one_item_sets, 0.5);

        let rules =
            generate_association_rules(&one_item_sets, [two_item_sets], 0.7);

        assert_eq!(rules.len(), 2);

        let bread_to_milk = rules
            .iter()
            .find(|rule| rule.antecedent == ["Bread"].into_iter().collect())
            .unwrap();

        assert_eq!(bread_to_milk.consequent, ["Milk"].into_iter().collect());
    }
}
