use crate::simulation::deck::Card;
use std::collections::HashSet;

pub fn combinations<'a, I>(cards: I, r: usize) -> Vec<HashSet<Card>>
where
    I: Iterator<Item = &'a Card>,
{
    let pool: Vec<Card> = cards.cloned().collect();
    let mut results = Vec::new();
    let n = pool.len();
    if n < r {
        return results;
    }
    let mut indices = Vec::from_iter(0..r);
    results.push(HashSet::from_iter(
        indices.clone().into_iter().map(|i| pool[i]),
    ));
    loop {
        let idx = (0..r).rev().find(|i| indices[*i] != i + n - r);
        match idx {
            Some(i) => {
                indices[i] += 1;
                for j in (i + 1)..r {
                    indices[j] = indices[j - 1] + 1;
                }
                results.push(HashSet::from_iter(
                    indices.clone().into_iter().map(|i| pool[i]),
                ));
            }
            None => break,
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::deck::Card;

    #[test]
    fn test_pairs() {
        let cards = vec![
            Card::from("ac"),
            Card::from("2d"),
            Card::from("3s"),
            Card::from("4h"),
        ];

        let results = combinations(cards.iter(), 2);
        assert_eq!(results.len(), 6);
        assert!(results.contains(&HashSet::from_iter(
            [Card::from("ac"), Card::from("2d")].into_iter()
        )));
        assert!(results.contains(&HashSet::from_iter(
            [Card::from("ac"), Card::from("3s")].into_iter()
        )));
        assert!(results.contains(&HashSet::from_iter(
            [Card::from("ac"), Card::from("4h")].into_iter()
        )));
        assert!(results.contains(&HashSet::from_iter(
            [Card::from("2d"), Card::from("3s")].into_iter()
        )));
        assert!(results.contains(&HashSet::from_iter(
            [Card::from("2d"), Card::from("4h")].into_iter()
        )));
        assert!(results.contains(&HashSet::from_iter(
            [Card::from("3s"), Card::from("4h")].into_iter()
        )));
    }
}
