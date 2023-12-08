use crate::simulation::combinatorics::combinations;
use crate::simulation::deck::Card;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Scorings {
    Fifteen(HashSet<Card>),
    Pair(HashSet<Card>),
    RunOfThree(HashSet<Card>),
    RunOfFour(HashSet<Card>),
    RunOfFive(HashSet<Card>),
    RunOfSix(HashSet<Card>),
    RunOfSeven(HashSet<Card>),
}

pub fn score_pegging(played: Vec<Card>) -> Vec<Scorings> {
    let mut scorings = Vec::new();

    /* Check for fifteen */
    if played.iter().map(|c| c.rank.value()).sum::<u8>() == 15 {
        scorings.push(Scorings::Fifteen(HashSet::from_iter(
            played.iter().cloned(),
        )));
    };

    /* Find pairs */
    let mut num_pairs = 0;
    let mut last_card: Option<Card> = None;
    for card in played.iter().rev() {
        if let Some(last_card_inner) = last_card {
            if card.rank == last_card_inner.rank {
                num_pairs += 1;
            } else {
                break;
            }
        }
        last_card = Some(*card);
    }
    let pairs = combinations(played.iter().rev().take(num_pairs + 1), 2);
    for pair in pairs.iter() {
        scorings.push(Scorings::Pair(HashSet::from_iter(pair.iter().cloned())));
    }

    /* Find runs */
    let mut maybe_run = None;
    for length in 3..(played.len() + 1) {
        let mut test: Vec<Card> = played
            .iter()
            .rev()
            .take(length)
            .cloned()
            .collect::<Vec<Card>>();
        test.sort();
        let ranks: Vec<usize> = test.iter().map(|x| x.rank.ordinal()).collect();
        if ranks.iter().cloned().eq((ranks[0])..(ranks[0] + length)) {
            maybe_run = Some(match length {
                3 => Scorings::RunOfThree(HashSet::from_iter(test.into_iter())),
                4 => Scorings::RunOfFour(HashSet::from_iter(test.into_iter())),
                5 => Scorings::RunOfFive(HashSet::from_iter(test.into_iter())),
                6 => Scorings::RunOfSix(HashSet::from_iter(test.into_iter())),
                7 => Scorings::RunOfSeven(HashSet::from_iter(test.into_iter())),
                _ => panic!("Too many cards!"),
            })
        }
    }

    if let Some(run_result) = maybe_run {
        scorings.push(run_result);
    }

    scorings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::deck::Card;

    #[test]
    fn test_score_pegging() {
        let tests: Vec<(&str, Vec<Card>, Vec<Scorings>)> = Vec::from([
            ("No cards played", Vec::new(), Vec::new()),
            (
                "Non-scoring card played",
                Vec::from([Card::from("as")]),
                Vec::new(),
            ),
            (
                "Fifteen",
                Vec::from([Card::from("7s"), Card::from("8c")]),
                Vec::from([Scorings::Fifteen(HashSet::from_iter(
                    [Card::from("7s"), Card::from("8c")].iter().cloned(),
                ))]),
            ),
            (
                "Pair",
                Vec::from([Card::from("as"), Card::from("ac")]),
                Vec::from([Scorings::Pair(HashSet::from_iter(
                    [Card::from("ac"), Card::from("as")].iter().cloned(),
                ))]),
            ),
            (
                "Pair royale",
                Vec::from([Card::from("as"), Card::from("ac"), Card::from("ad")]),
                Vec::from([
                    Scorings::Pair(HashSet::from_iter(
                        [Card::from("ac"), Card::from("as")].iter().cloned(),
                    )),
                    Scorings::Pair(HashSet::from_iter(
                        [Card::from("ac"), Card::from("ad")].iter().cloned(),
                    )),
                    Scorings::Pair(HashSet::from_iter(
                        [Card::from("as"), Card::from("ad")].iter().cloned(),
                    )),
                ]),
            ),
            (
                "Double pair royale",
                Vec::from([
                    Card::from("as"),
                    Card::from("ac"),
                    Card::from("ad"),
                    Card::from("ah"),
                ]),
                Vec::from([
                    Scorings::Pair(HashSet::from_iter(
                        [Card::from("as"), Card::from("ac")].iter().cloned(),
                    )),
                    Scorings::Pair(HashSet::from_iter(
                        [Card::from("as"), Card::from("ad")].iter().cloned(),
                    )),
                    Scorings::Pair(HashSet::from_iter(
                        [Card::from("as"), Card::from("ah")].iter().cloned(),
                    )),
                    Scorings::Pair(HashSet::from_iter(
                        [Card::from("ac"), Card::from("ad")].iter().cloned(),
                    )),
                    Scorings::Pair(HashSet::from_iter(
                        [Card::from("ac"), Card::from("ah")].iter().cloned(),
                    )),
                    Scorings::Pair(HashSet::from_iter(
                        [Card::from("ad"), Card::from("ah")].iter().cloned(),
                    )),
                ]),
            ),
            (
                "Run of three",
                Vec::from([
                    Card::from("as"),
                    Card::from("2s"),
                    Card::from("3s"),
                ]),
                Vec::from([Scorings::RunOfThree(HashSet::from_iter(
                    [Card::from("as"), Card::from("2s"), Card::from("3s")]
                        .iter()
                        .cloned(),
                ))]),
            ),
            (
                "Run of four",
                Vec::from([
                    Card::from("as"),
                    Card::from("2s"),
                    Card::from("3s"),
                    Card::from("4s"),
                ]),
                Vec::from([Scorings::RunOfFour(HashSet::from_iter(
                    [
                        Card::from("as"),
                        Card::from("2s"),
                        Card::from("3s"),
                        Card::from("4s"),
                    ]
                    .iter()
                    .cloned(),
                ))]),
            ),
            (
                "Run of five",
                Vec::from([
                    Card::from("as"),
                    Card::from("2s"),
                    Card::from("3s"),
                    Card::from("4s"),
                    Card::from("5s"),
                ]),
                Vec::from([Scorings::RunOfFive(HashSet::from_iter(
                    [
                        Card::from("as"),
                        Card::from("2s"),
                        Card::from("3s"),
                        Card::from("4s"),
                        Card::from("5s"),
                    ]
                    .iter()
                    .cloned(),
                ))]),
            ),
            (
                "Run of six",
                Vec::from([
                    Card::from("as"),
                    Card::from("2s"),
                    Card::from("3s"),
                    Card::from("4s"),
                    Card::from("5s"),
                    Card::from("6s"),
                ]),
                Vec::from([Scorings::RunOfSix(HashSet::from_iter(
                    [
                        Card::from("as"),
                        Card::from("2s"),
                        Card::from("3s"),
                        Card::from("4s"),
                        Card::from("5s"),
                        Card::from("6s"),
                    ]
                    .iter()
                    .cloned(),
                ))]),
            ),
        ]);

        for (description, test, expected) in tests {
            for shuffled in combinations(test.iter(), test.len()) {
                let scorings = score_pegging(Vec::from_iter(shuffled.iter().cloned()));
                for scoring in expected.iter().cloned() {
                    assert!(
                        scorings.contains(&scoring),
                        "{}, {:?}, {:?}",
                        description,
                        scoring,
                        scorings,
                    );
                }
            }
        }
    }
}
