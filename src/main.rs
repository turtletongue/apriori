use std::{
    cmp::Ordering,
    collections::HashSet,
    error::Error,
    fmt::{self, Display},
    fs::File,
    io::{BufRead as _, BufReader},
    time::Instant,
};

use apriori::{self, KItemSet, OneItemSet};
use clap::{Parser, ValueEnum};
use ordered_float::NotNan;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let rows: Vec<_> = BufReader::new(File::open(args.file_path)?)
        .lines()
        .collect::<Result<_, _>>()?;

    let data: Vec<_> = rows
        .iter()
        .map(|row| {
            row.split(',').map(str::trim).collect::<HashSet<&str>>()
        })
        .collect();

    let start = Instant::now();

    let one_item_sets = apriori::generate_one_item_sets(&data, args.support);
    let two_item_sets =
        apriori::generate_two_item_sets(&data, &one_item_sets, args.support);
    let mut last_k_item_sets = vec![two_item_sets];
    let mut i = 1;

    loop {
        let sets = apriori::generate_next_sets(
            &data,
            &last_k_item_sets[i - 1],
            &one_item_sets,
            args.support,
        );

        if sets.is_empty() {
            break;
        }

        last_k_item_sets.push(sets);
        i += 1;
    }

    let end =  start.elapsed();

    let mut results: Vec<_> = last_k_item_sets
        .into_iter()
        .flat_map(|sets| sets.into_iter().map(ResultingSet::KItem))
        .chain(one_item_sets.into_iter().map(ResultingSet::OneItem))
        .collect();

    results.sort_by(|a, b| match args.ordering {
        ResultsOrdering::Support => a.support().cmp(&b.support()).reverse(),
        ResultsOrdering::Lexicographic => {
            if let (ResultingSet::OneItem(a), ResultingSet::OneItem(b)) = (a, b)
            {
                a.item.cmp(b.item)
            } else {
                Ordering::Equal
            }
        }
    });

    for result in &results {
        println!("{result}");
    }

    println!("\nCount: {}", results.len());
    println!("Elapsed: {}ms", end.as_millis());

    Ok(())
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to data file
    #[arg(short, long, default_value = "baskets.csv")]
    file_path: String,

    /// Results ordering
    #[arg(short, long, value_enum, default_value_t = ResultsOrdering::Support)]
    ordering: ResultsOrdering,

    /// Support threshold
    #[arg(short, long)]
    support: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
#[value(rename_all = "kebab-case")]
enum ResultsOrdering {
    Lexicographic,
    Support,
}

#[derive(Clone, Debug)]
enum ResultingSet<T> {
    OneItem(OneItemSet<T>),
    KItem(KItemSet<T>),
}

impl<T> ResultingSet<T> {
    const fn support(&self) -> NotNan<f64> {
        match self {
            Self::OneItem(one_item_set) => one_item_set.support,
            Self::KItem(k_item_set) => k_item_set.support,
        }
    }
}

impl<T> Display for ResultingSet<T>
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
