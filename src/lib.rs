use std::{
    collections::{HashMap, HashSet}, hash::Hash,
};

use ordered_float::NotNan;

pub use crate::sets::{KItemSet, OneItemSet};

mod sets;

#[must_use]
pub fn generate_one_item_sets<'a, T>(
    data: &[HashSet<&'a T>],
    min_support: f64,
) -> Vec<OneItemSet<&'a T>>
where
    T: Eq + Hash + PartialEq + ?Sized,
{
    let map = data.iter().enumerate().fold(
        HashMap::new(),
        |mut map, (i, transaction)| {
            for &item in transaction {
                map.entry(item)
                    .and_modify(|(count, indexes): &mut (_, HashSet<_>)| {
                        *count += 1;
                        indexes.insert(i);
                    })
                    .or_insert_with(|| (1, [i].into_iter().collect()));
            }

            map
        },
    );

    map.into_iter()
        .map(|(item, (count, indexes))| {
            let support = NotNan::new((count as f64) / (data.len() as f64))
                .expect("never NaN");

            (item, support, (count, indexes))
        })
        .filter(|(_, support, _)| support.into_inner() >= min_support)
        .map(|(item, support, (count, indexes))| {
            OneItemSet::new(item, count, support, indexes)
        })
        .collect()
}

#[must_use]
pub fn generate_two_item_sets<'a, T>(
    data: &[HashSet<&'a T>],
    one_item_sets: &[OneItemSet<&'a T>],
    min_support: f64,
) -> Vec<KItemSet<&'a T>>
where
    T: Eq + Hash + PartialEq + ?Sized,
{
    one_item_sets
        .iter()
        .enumerate()
        .flat_map(|(i, a)| {
            one_item_sets
                .iter()
                .skip(i + 1)
                .map(|b| {
                    let merged: HashSet<_> =
                        [a.item, b.item].into_iter().collect();

                    let row_indexes: HashSet<_> = a
                        .row_indexes
                        .intersection(&b.row_indexes)
                        .copied()
                        .collect();

                    KItemSet::new(merged, data, row_indexes)
                })
                .filter(|set| set.support.into_inner() >= min_support)
        })
        .collect()
}

#[must_use]
pub fn generate_next_sets<'a, T>(
    data: &[HashSet<&'a T>],
    previous_sets: &[KItemSet<&'a T>],
    one_item_sets: &[OneItemSet<&'a T>],
    min_support: f64,
) -> Vec<KItemSet<&'a T>>
where
    T: Eq + Hash + PartialEq + ?Sized,
{
    previous_sets
        .iter()
        .flat_map(|previous_set| {
            one_item_sets
                .iter()
                .filter(|one_item_set| {
                    !previous_set.items.contains(one_item_set.item)
                })
                .map(|one_item_set| {
                    let merged: HashSet<_> = previous_set
                        .items
                        .iter()
                        .copied()
                        .chain([one_item_set.item])
                        .collect();

                    let row_indexes: HashSet<_> = previous_set
                        .row_indexes
                        .intersection(&one_item_set.row_indexes)
                        .copied()
                        .collect();

                    KItemSet::new(merged, data, row_indexes)
                })
                .filter(|set| set.support.into_inner() >= min_support)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_data() -> Vec<HashSet<&'static str>> {
        vec![
            HashSet::from(["Bread", "Butter", "Milk"]),
            HashSet::from(["Bread", "Butter"]),
            HashSet::from(["Bread", "Milk"]),
            HashSet::from(["Butter", "Milk"]),
            HashSet::from(["Bread", "Milk"]),
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

        let bread_butter = sets
            .iter()
            .find(|set| {
                set.items.is_superset(&["Bread", "Milk"].into_iter().collect())
            })
            .unwrap();

        assert_eq!(bread_butter.support_count, 3);
        assert_eq!(bread_butter.support, NotNan::new(0.6).unwrap());
    }

    #[test]
    fn three_item_sets() {
        let data = get_data();
        let one_item_sets = generate_one_item_sets(&data, 0.5);
        let two_item_sets = generate_two_item_sets(&data, &one_item_sets, 0.5);

        let sets =
            generate_next_sets(&data, &two_item_sets, &one_item_sets, 0.5);

        assert_eq!(sets.len(), 0);
    }
}
