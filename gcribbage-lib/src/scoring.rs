use crate::combinatorics::combinations;
use crate::deck::{Card, Rank, Suit};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PeggingScorings {
    Fifteen,
    Pair(HashSet<Card>),
    RunOfThree,
    RunOfFour,
    RunOfFive,
    RunOfSix,
    RunOfSeven,
    ThirtyOne,
    LastCard,
}

impl PeggingScorings {
    pub fn value(&self) -> u8 {
        use PeggingScorings::*;
        match self {
            Fifteen => 2,
            Pair(_) => 2,
            RunOfThree => 3,
            RunOfFour => 4,
            RunOfFive => 5,
            RunOfSix => 6,
            RunOfSeven => 7,
            ThirtyOne => 2,
            LastCard => 1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HandScorings {
    Fifteen(HashSet<Card>),
    Pair(HashSet<Card>),
    RunOfThree(HashSet<Card>),
    RunOfFour(HashSet<Card>),
    RunOfFive(HashSet<Card>),
    FourCardFlush(HashSet<Card>),
    FiveCardFlush(HashSet<Card>),
    Nobs(Card),
}

impl HandScorings {
    pub fn value(&self) -> u8 {
        use HandScorings::*;
        match self {
            Fifteen(_) => 2,
            Pair(_) => 2,
            RunOfThree(_) => 3,
            RunOfFour(_) => 4,
            RunOfFive(_) => 5,
            FourCardFlush(_) => 4,
            FiveCardFlush(_) => 5,
            Nobs(_) => 1,
        }
    }
}

pub fn score_pegging(played: Vec<Card>) -> Vec<PeggingScorings> {
    let mut scorings = Vec::new();

    /* Check for fifteen, or thirty-one */
    match played.iter().map(|c| c.rank.value()).sum::<u8>() {
        15 => scorings.push(PeggingScorings::Fifteen),
        31 => scorings.push(PeggingScorings::ThirtyOne),
        _ => (),
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
        scorings.push(PeggingScorings::Pair(HashSet::from_iter(
            pair.iter().cloned(),
        )));
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
                3 => PeggingScorings::RunOfThree,
                4 => PeggingScorings::RunOfFour,
                5 => PeggingScorings::RunOfFive,
                6 => PeggingScorings::RunOfSix,
                7 => PeggingScorings::RunOfSeven,
                _ => panic!("Too many cards!"),
            })
        }
    }

    if let Some(run_result) = maybe_run {
        scorings.push(run_result);
    }

    scorings
}

pub fn score_hand(hand: &HashSet<Card>, up_card: Card) -> Vec<HandScorings> {
    let mut set: Vec<Card> = hand.iter().copied().collect();

    set.push(up_card);
    let mut scorings = Vec::new();

    let mut skip_runs = false;
    let mut skip_flush = false;
    for pick in (2..=5).rev() {
        let mut run_found = false;
        let mut flush_found = false;
        for combo in combinations(set.iter(), pick) {
            /* Find fifteen */
            if combo.iter().map(|c| c.rank.value()).sum::<u8>() == 15 {
                scorings.push(HandScorings::Fifteen(HashSet::from_iter(
                    combo.iter().cloned(),
                )));
            }

            /* Find pair */
            if pick == 2 {
                let mut cards = combo.iter();
                let left = cards.next().unwrap();
                let right = cards.next().unwrap();
                if left.rank == right.rank {
                    scorings.push(HandScorings::Pair(HashSet::from_iter(
                        combo.iter().cloned(),
                    )));
                }
            }

            /* Find runs */
            if pick > 2 && !skip_runs {
                let mut sorted = combo.iter().cloned().collect::<Vec<Card>>();
                sorted.sort();
                let ranks = sorted
                    .iter()
                    .map(|c| c.rank.ordinal())
                    .collect::<Vec<usize>>();
                if ranks.iter().cloned().eq((ranks[0])..(ranks[0] + pick)) {
                    let result = HashSet::from_iter(combo.iter().cloned());
                    scorings.push(match pick {
                        3 => HandScorings::RunOfThree(result),
                        4 => HandScorings::RunOfFour(result),
                        5 => HandScorings::RunOfFive(result),
                        _ => panic!("Run of wrong size!"),
                    });
                    run_found = true;
                }
            }

            /* Find flush */
            if pick > 3 && !skip_flush {
                for suit in [Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds] {
                    let matching_cards = combo
                        .iter()
                        .filter(|c| c.suit == suit)
                        .cloned()
                        .collect::<HashSet<Card>>();
                    match matching_cards.len() {
                        4 => {
                            scorings.push(HandScorings::FourCardFlush(matching_cards));
                            flush_found = true;
                        }
                        5 => {
                            scorings.push(HandScorings::FiveCardFlush(matching_cards));
                            flush_found = true;
                        }
                        _ => (),
                    }
                }
            }
        }
        if run_found {
            skip_runs = true;
        }

        if flush_found {
            skip_flush = true;
        }
    }

    /* Find nobs */
    for jack in set.iter().filter(|c| c.rank == Rank::Jack) {
        if jack.suit == up_card.suit {
            scorings.push(HandScorings::Nobs(*jack));
            break;
        }
    }
    scorings
}

pub fn score_crib(hand: &HashSet<Card>, up_card: Card) -> Vec<HandScorings> {
    score_hand(hand, up_card)
        .iter()
        .filter(|s| !matches!(s, HandScorings::FourCardFlush(_)))
        .cloned()
        .collect()
}

#[cfg(test)]
mod hand_scoring {
    use super::*;
    use crate::deck::Card;

    fn contains(hand: &HashSet<Card>, up_card: &Card, expected: &Vec<HandScorings>) -> bool {
        let scorings = score_hand(hand, *up_card);
        for scoring in expected.iter().cloned() {
            if !scorings.contains(&scoring) {
                return false;
            }
        }
        true
    }

    #[test]
    fn test_non_scoring_hand() {
        let hand = HashSet::from_iter(vec![
            Card::from("Ts"),
            Card::from("Qh"),
            Card::from("7h"),
            Card::from("Ad"),
        ]);
        let up_card = Card::from("Ks");
        let result = score_hand(&hand, up_card);
        assert_eq!(0, result.len());
    }

    #[test]
    fn test_score_two_card_fifteens() {
        let tests: Vec<(&str, HashSet<Card>, Card, Vec<HandScorings>)> = Vec::from([
            (
                "7-8 Fifteen",
                HashSet::from_iter(vec![
                    Card::from("7s"),
                    Card::from("8h"),
                    Card::from("Th"),
                    Card::from("Qd"),
                ]),
                Card::from("Ks"),
                Vec::from([HandScorings::Fifteen(HashSet::from_iter(
                    [Card::from("7s"), Card::from("8h")].iter().cloned(),
                ))]),
            ),
            (
                "9-6 Fifteen",
                HashSet::from_iter(vec![
                    Card::from("9s"),
                    Card::from("6h"),
                    Card::from("Th"),
                    Card::from("Qd"),
                ]),
                Card::from("Ks"),
                Vec::from([HandScorings::Fifteen(HashSet::from_iter(
                    [Card::from("9s"), Card::from("6h")].iter().cloned(),
                ))]),
            ),
            (
                "10-5 Fifteen",
                HashSet::from_iter(vec![
                    Card::from("Ts"),
                    Card::from("5h"),
                    Card::from("9h"),
                    Card::from("Qd"),
                ]),
                Card::from("Ks"),
                Vec::from([HandScorings::Fifteen(HashSet::from_iter(
                    [Card::from("Ts"), Card::from("5h")].iter().cloned(),
                ))]),
            ),
            (
                "J-5 Fifteen",
                HashSet::from_iter(vec![
                    Card::from("Js"),
                    Card::from("5h"),
                    Card::from("9h"),
                    Card::from("Td"),
                ]),
                Card::from("Ks"),
                Vec::from([HandScorings::Fifteen(HashSet::from_iter(
                    [Card::from("Js"), Card::from("5h")].iter().cloned(),
                ))]),
            ),
            (
                "Q-5 Fifteen",
                HashSet::from_iter(vec![
                    Card::from("Qs"),
                    Card::from("5h"),
                    Card::from("9h"),
                    Card::from("Td"),
                ]),
                Card::from("Ks"),
                Vec::from([HandScorings::Fifteen(HashSet::from_iter(
                    [Card::from("Qs"), Card::from("5h")].iter().cloned(),
                ))]),
            ),
            (
                "K-5 Fifteen",
                HashSet::from_iter(vec![
                    Card::from("Ks"),
                    Card::from("5h"),
                    Card::from("9h"),
                    Card::from("Td"),
                ]),
                Card::from("Qs"),
                Vec::from([HandScorings::Fifteen(HashSet::from_iter(
                    [Card::from("Ks"), Card::from("5h")].iter().cloned(),
                ))]),
            ),
            (
                "Multiple Fifteens",
                HashSet::from_iter(vec![
                    Card::from("Ks"),
                    Card::from("5h"),
                    Card::from("Qh"),
                    Card::from("Td"),
                ]),
                Card::from("Qs"),
                Vec::from([
                    HandScorings::Fifteen(HashSet::from_iter(
                        [Card::from("Ks"), Card::from("5h")].iter().cloned(),
                    )),
                    HandScorings::Fifteen(HashSet::from_iter(
                        [Card::from("Qh"), Card::from("5h")].iter().cloned(),
                    )),
                    HandScorings::Fifteen(HashSet::from_iter(
                        [Card::from("Td"), Card::from("5h")].iter().cloned(),
                    )),
                    HandScorings::Fifteen(HashSet::from_iter(
                        [Card::from("Qs"), Card::from("5h")].iter().cloned(),
                    )),
                ]),
            ),
        ]);

        for (description, hand, up_card, expected) in tests {
            assert!(contains(&hand, &up_card, &expected,), "{}", description,);
        }
    }

    #[test]
    fn test_score_three_card_fifteens() {
        let tests: Vec<(&str, HashSet<Card>, Card, Vec<HandScorings>)> = Vec::from([
            (
                "T-4-A Fifteen",
                HashSet::from_iter(vec![
                    Card::from("As"),
                    Card::from("4h"),
                    Card::from("9h"),
                    Card::from("Td"),
                ]),
                Card::from("7s"),
                Vec::from([HandScorings::Fifteen(HashSet::from_iter(
                    [Card::from("As"), Card::from("4h"), Card::from("Td")]
                        .iter()
                        .cloned(),
                ))]),
            ),
            (
                "Multiple three card Fifteens",
                HashSet::from_iter(vec![
                    Card::from("As"),
                    Card::from("4h"),
                    Card::from("9h"),
                    Card::from("Td"),
                ]),
                Card::from("Ks"),
                Vec::from([
                    HandScorings::Fifteen(HashSet::from_iter(
                        [Card::from("As"), Card::from("4h"), Card::from("Td")]
                            .iter()
                            .cloned(),
                    )),
                    HandScorings::Fifteen(HashSet::from_iter(
                        [Card::from("As"), Card::from("4h"), Card::from("Ks")]
                            .iter()
                            .cloned(),
                    )),
                ]),
            ),
        ]);
        for (description, hand, up_card, expected) in tests {
            assert!(contains(&hand, &up_card, &expected,), "{}", description,);
        }
    }

    #[test]
    fn four_card_fifteens() {
        let tests: Vec<(&str, HashSet<Card>, Card, Vec<HandScorings>)> = Vec::from([
            (
                "T-3-A-A Fifteen",
                HashSet::from_iter(vec![
                    Card::from("As"),
                    Card::from("Ah"),
                    Card::from("3h"),
                    Card::from("Td"),
                ]),
                Card::from("7s"),
                Vec::from([HandScorings::Fifteen(HashSet::from_iter(
                    [
                        Card::from("As"),
                        Card::from("Ah"),
                        Card::from("3h"),
                        Card::from("Td"),
                    ]
                    .iter()
                    .cloned(),
                ))]),
            ),
            (
                "Multiple four card Fifteens",
                HashSet::from_iter(vec![
                    Card::from("As"),
                    Card::from("Ah"),
                    Card::from("3h"),
                    Card::from("Td"),
                ]),
                Card::from("Ks"),
                Vec::from([
                    HandScorings::Fifteen(HashSet::from_iter(
                        [
                            Card::from("As"),
                            Card::from("Ah"),
                            Card::from("3h"),
                            Card::from("Td"),
                        ]
                        .iter()
                        .cloned(),
                    )),
                    HandScorings::Fifteen(HashSet::from_iter(
                        [
                            Card::from("As"),
                            Card::from("Ah"),
                            Card::from("3h"),
                            Card::from("Ks"),
                        ]
                        .iter()
                        .cloned(),
                    )),
                ]),
            ),
        ]);

        for (description, hand, up_card, expected) in tests {
            assert!(contains(&hand, &up_card, &expected,), "{}", description,);
        }
    }

    #[test]
    fn five_card_fifteens() {
        let tests: Vec<(&str, HashSet<Card>, Card, Vec<HandScorings>)> = Vec::from([
            (
                "5-4-2-2-2 Fifteen",
                HashSet::from_iter(vec![
                    Card::from("5s"),
                    Card::from("4h"),
                    Card::from("2h"),
                    Card::from("2d"),
                ]),
                Card::from("2s"),
                Vec::from([HandScorings::Fifteen(HashSet::from_iter(
                    [
                        Card::from("5s"),
                        Card::from("4h"),
                        Card::from("2h"),
                        Card::from("2d"),
                        Card::from("2s"),
                    ]
                    .iter()
                    .cloned(),
                ))]),
            ),
            (
                "6-5-2-A-A Fifteen",
                HashSet::from_iter(vec![
                    Card::from("6s"),
                    Card::from("5h"),
                    Card::from("2h"),
                    Card::from("Ad"),
                ]),
                Card::from("As"),
                Vec::from([HandScorings::Fifteen(HashSet::from_iter(
                    [
                        Card::from("6s"),
                        Card::from("5h"),
                        Card::from("2h"),
                        Card::from("Ad"),
                        Card::from("As"),
                    ]
                    .iter()
                    .cloned(),
                ))]),
            ),
        ]);
        for (description, hand, up_card, expected) in tests {
            assert!(contains(&hand, &up_card, &expected,), "{}", description,);
        }
    }

    #[test]
    fn test_pairs() {
        let tests: Vec<(&str, HashSet<Card>, Card, Vec<HandScorings>)> = Vec::from([
            (
                "Single pair",
                HashSet::from_iter(vec![
                    Card::from("2s"),
                    Card::from("2c"),
                    Card::from("Ah"),
                    Card::from("Th"),
                ]),
                Card::from("Ks"),
                Vec::from([HandScorings::Pair(HashSet::from_iter([
                    Card::from("2s"),
                    Card::from("2c"),
                ]))]),
            ),
            (
                "Two distinct pairs",
                HashSet::from_iter(vec![
                    Card::from("2s"),
                    Card::from("2c"),
                    Card::from("Ah"),
                    Card::from("Ad"),
                ]),
                Card::from("Ks"),
                Vec::from([
                    HandScorings::Pair(HashSet::from_iter([Card::from("2s"), Card::from("2c")])),
                    HandScorings::Pair(HashSet::from_iter([Card::from("Ah"), Card::from("Ad")])),
                ]),
            ),
            (
                "Pair royale",
                HashSet::from_iter(vec![
                    Card::from("2s"),
                    Card::from("2c"),
                    Card::from("2h"),
                    Card::from("Ad"),
                ]),
                Card::from("Ks"),
                Vec::from([
                    HandScorings::Pair(HashSet::from_iter([Card::from("2s"), Card::from("2c")])),
                    HandScorings::Pair(HashSet::from_iter([Card::from("2s"), Card::from("2h")])),
                    HandScorings::Pair(HashSet::from_iter([Card::from("2c"), Card::from("2h")])),
                ]),
            ),
            (
                "Double pair royale",
                HashSet::from_iter(vec![
                    Card::from("2s"),
                    Card::from("2c"),
                    Card::from("2h"),
                    Card::from("2d"),
                ]),
                Card::from("Ks"),
                Vec::from([
                    HandScorings::Pair(HashSet::from_iter([Card::from("2s"), Card::from("2c")])),
                    HandScorings::Pair(HashSet::from_iter([Card::from("2s"), Card::from("2h")])),
                    HandScorings::Pair(HashSet::from_iter([Card::from("2s"), Card::from("2d")])),
                    HandScorings::Pair(HashSet::from_iter([Card::from("2c"), Card::from("2h")])),
                    HandScorings::Pair(HashSet::from_iter([Card::from("2c"), Card::from("2d")])),
                    HandScorings::Pair(HashSet::from_iter([Card::from("2h"), Card::from("2d")])),
                ]),
            ),
        ]);

        for (description, hand, up_card, expected) in tests {
            assert!(contains(&hand, &up_card, &expected,), "{}", description,);
        }
    }

    #[test]
    fn runs_of_three() {
        let tests: Vec<(&str, HashSet<Card>, Card, Vec<HandScorings>)> = Vec::from([
            (
                "Single run of three",
                HashSet::from_iter(vec![
                    Card::from("As"),
                    Card::from("2s"),
                    Card::from("3d"),
                    Card::from("Tc"),
                ]),
                Card::from("Ks"),
                Vec::from([HandScorings::RunOfThree(HashSet::from_iter([
                    Card::from("As"),
                    Card::from("2s"),
                    Card::from("3d"),
                ]))]),
            ),
            (
                "Double run of three",
                HashSet::from_iter(vec![
                    Card::from("As"),
                    Card::from("2s"),
                    Card::from("3d"),
                    Card::from("Ac"),
                ]),
                Card::from("Ks"),
                Vec::from([
                    HandScorings::RunOfThree(HashSet::from_iter([
                        Card::from("As"),
                        Card::from("2s"),
                        Card::from("3d"),
                    ])),
                    HandScorings::RunOfThree(HashSet::from_iter([
                        Card::from("Ac"),
                        Card::from("2s"),
                        Card::from("3d"),
                    ])),
                ]),
            ),
            (
                "Triple run of three",
                HashSet::from_iter(vec![
                    Card::from("As"),
                    Card::from("2s"),
                    Card::from("3d"),
                    Card::from("Ac"),
                ]),
                Card::from("Ah"),
                Vec::from([
                    HandScorings::RunOfThree(HashSet::from_iter([
                        Card::from("As"),
                        Card::from("2s"),
                        Card::from("3d"),
                    ])),
                    HandScorings::RunOfThree(HashSet::from_iter([
                        Card::from("Ac"),
                        Card::from("2s"),
                        Card::from("3d"),
                    ])),
                    HandScorings::RunOfThree(HashSet::from_iter([
                        Card::from("Ah"),
                        Card::from("2s"),
                        Card::from("3d"),
                    ])),
                ]),
            ),
        ]);
        for (description, hand, up_card, expected) in tests {
            assert!(contains(&hand, &up_card, &expected,), "{}", description,);
        }
    }

    #[test]
    fn runs_of_four() {
        let tests: Vec<(&str, HashSet<Card>, Card, Vec<HandScorings>, Vec<HandScorings>)> =
            Vec::from([
                (
                    "Single run of four",
                    HashSet::from_iter(vec![
                        Card::from("As"),
                        Card::from("2s"),
                        Card::from("3d"),
                        Card::from("4c"),
                    ]),
                    Card::from("Ks"),
                    Vec::from([HandScorings::RunOfFour(HashSet::from_iter([
                        Card::from("As"),
                        Card::from("2s"),
                        Card::from("3d"),
                        Card::from("4c"),
                    ]))]),
                    Vec::from([
                        HandScorings::RunOfThree(HashSet::from_iter([
                            Card::from("As"),
                            Card::from("2s"),
                            Card::from("3d"),
                        ])),
                        HandScorings::RunOfThree(HashSet::from_iter([
                            Card::from("2s"),
                            Card::from("3d"),
                            Card::from("4c"),
                        ])),
                    ]),
                ),
                (
                    "Double run of four",
                    HashSet::from_iter(vec![
                        Card::from("As"),
                        Card::from("2s"),
                        Card::from("3d"),
                        Card::from("4c"),
                    ]),
                    Card::from("Ac"),
                    Vec::from([
                        HandScorings::RunOfFour(HashSet::from_iter([
                            Card::from("As"),
                            Card::from("2s"),
                            Card::from("3d"),
                            Card::from("4c"),
                        ])),
                        HandScorings::RunOfFour(HashSet::from_iter([
                            Card::from("Ac"),
                            Card::from("2s"),
                            Card::from("3d"),
                            Card::from("4c"),
                        ])),
                    ]),
                    Vec::from([
                        HandScorings::RunOfThree(HashSet::from_iter([
                            Card::from("As"),
                            Card::from("2s"),
                            Card::from("3d"),
                        ])),
                        HandScorings::RunOfThree(HashSet::from_iter([
                            Card::from("Ac"),
                            Card::from("2s"),
                            Card::from("3d"),
                        ])),
                        HandScorings::RunOfThree(HashSet::from_iter([
                            Card::from("2s"),
                            Card::from("3d"),
                            Card::from("4c"),
                        ])),
                    ]),
                ),
            ]);
        for (description, hand, up_card, expected, unexpecteds) in tests {
            assert!(contains(&hand, &up_card, &expected,), "{}", description,);
            for unexpected in unexpecteds {
                assert!(
                    !contains(&hand, &up_card, &Vec::from([unexpected.clone()])),
                    "{} {:?}",
                    description,
                    unexpected
                );
            }
        }
    }

    #[test]
    fn runs_of_five() {
        let tests: Vec<(&str, HashSet<Card>, Card, Vec<HandScorings>, Vec<HandScorings>)> =
            Vec::from([(
                "Run of five",
                HashSet::from_iter(vec![
                    Card::from("As"),
                    Card::from("2s"),
                    Card::from("3d"),
                    Card::from("4c"),
                ]),
                Card::from("5s"),
                Vec::from([HandScorings::RunOfFive(HashSet::from_iter([
                    Card::from("As"),
                    Card::from("2s"),
                    Card::from("3d"),
                    Card::from("4c"),
                    Card::from("5s"),
                ]))]),
                Vec::from([
                    HandScorings::RunOfFour(HashSet::from_iter([
                        Card::from("As"),
                        Card::from("2s"),
                        Card::from("3d"),
                        Card::from("4c"),
                    ])),
                    HandScorings::RunOfFour(HashSet::from_iter([
                        Card::from("2s"),
                        Card::from("3d"),
                        Card::from("4c"),
                        Card::from("5s"),
                    ])),
                    HandScorings::RunOfThree(HashSet::from_iter([
                        Card::from("As"),
                        Card::from("2s"),
                        Card::from("3d"),
                    ])),
                    HandScorings::RunOfThree(HashSet::from_iter([
                        Card::from("2s"),
                        Card::from("3d"),
                        Card::from("4c"),
                    ])),
                    HandScorings::RunOfThree(HashSet::from_iter([
                        Card::from("3d"),
                        Card::from("4c"),
                        Card::from("5s"),
                    ])),
                ]),
            )]);
        for (description, hand, up_card, expected, unexpecteds) in tests {
            assert!(contains(&hand, &up_card, &expected,), "{}", description,);
            for unexpected in unexpecteds {
                assert!(
                    !contains(&hand, &up_card, &Vec::from([unexpected.clone()])),
                    "{} {:?}",
                    description,
                    unexpected
                );
            }
        }
    }

    #[test]
    fn four_card_flush() {
        let tests: Vec<(&str, HashSet<Card>, Card, Vec<HandScorings>)> = Vec::from([(
            "Four card flush",
            HashSet::from_iter(vec![
                Card::from("As"),
                Card::from("2s"),
                Card::from("3s"),
                Card::from("4s"),
            ]),
            Card::from("5c"),
            Vec::from([HandScorings::FourCardFlush(HashSet::from_iter([
                Card::from("As"),
                Card::from("2s"),
                Card::from("3s"),
                Card::from("4s"),
            ]))]),
        )]);
        for (description, hand, up_card, expected) in tests {
            assert!(contains(&hand, &up_card, &expected,), "{}", description,);
        }
    }

    #[test]
    fn five_card_flush() {
        let tests: Vec<(&str, HashSet<Card>, Card, Vec<HandScorings>, Vec<HandScorings>)> =
            Vec::from([(
                "Five card flush",
                HashSet::from_iter(vec![
                    Card::from("As"),
                    Card::from("2s"),
                    Card::from("3s"),
                    Card::from("4s"),
                ]),
                Card::from("5s"),
                Vec::from([HandScorings::FiveCardFlush(HashSet::from_iter([
                    Card::from("As"),
                    Card::from("2s"),
                    Card::from("3s"),
                    Card::from("4s"),
                    Card::from("5s"),
                ]))]),
                Vec::from([
                    HandScorings::FourCardFlush(HashSet::from_iter([
                        Card::from("As"),
                        Card::from("2s"),
                        Card::from("3s"),
                        Card::from("4s"),
                    ])),
                    HandScorings::FourCardFlush(HashSet::from_iter([
                        Card::from("2s"),
                        Card::from("3s"),
                        Card::from("4s"),
                        Card::from("5s"),
                    ])),
                ]),
            )]);
        for (description, hand, up_card, expected, unexpecteds) in tests {
            assert!(contains(&hand, &up_card, &expected,), "{}", description,);
            for unexpected in unexpecteds {
                assert!(
                    !contains(&hand, &up_card, &Vec::from([unexpected.clone()])),
                    "{} {:?}",
                    description,
                    unexpected
                );
            }
        }
    }

    #[test]
    fn nobs() {
        let tests: Vec<(&str, HashSet<Card>, Card, Vec<HandScorings>)> = Vec::from([
            (
                "Nobs spades",
                HashSet::from_iter(vec![
                    Card::from("Js"),
                    Card::from("2c"),
                    Card::from("3h"),
                    Card::from("4d"),
                ]),
                Card::from("5s"),
                Vec::from([HandScorings::Nobs(Card::from("Js"))]),
            ),
            (
                "Nobs hearts",
                HashSet::from_iter(vec![
                    Card::from("Jh"),
                    Card::from("2c"),
                    Card::from("3s"),
                    Card::from("4d"),
                ]),
                Card::from("5h"),
                Vec::from([HandScorings::Nobs(Card::from("Jh"))]),
            ),
            (
                "Nobs clubs",
                HashSet::from_iter(vec![
                    Card::from("Jc"),
                    Card::from("2s"),
                    Card::from("3h"),
                    Card::from("4d"),
                ]),
                Card::from("5c"),
                Vec::from([HandScorings::Nobs(Card::from("Jc"))]),
            ),
            (
                "Nobs diamonds",
                HashSet::from_iter(vec![
                    Card::from("Jd"),
                    Card::from("2c"),
                    Card::from("3h"),
                    Card::from("4c"),
                ]),
                Card::from("5d"),
                Vec::from([HandScorings::Nobs(Card::from("Jd"))]),
            ),
        ]);
        for (description, hand, up_card, expected) in tests {
            assert!(contains(&hand, &up_card, &expected,), "{}", description,);
        }
    }
}

#[cfg(test)]
mod crib_scoring {
    use super::*;
    use crate::deck::Card;

    fn contains(hand: &HashSet<Card>, up_card: &Card, expected: &Vec<HandScorings>) -> bool {
        let scorings = score_crib(hand, *up_card);
        for scoring in expected.iter().cloned() {
            if !scorings.contains(&scoring) {
                return false;
            }
        }
        true
    }

    #[test]
    fn four_card_flush_not_allowed() {
        let tests: Vec<(&str, HashSet<Card>, Card, Vec<HandScorings>)> = Vec::from([(
            "Four card flush",
            HashSet::from_iter(vec![
                Card::from("As"),
                Card::from("2s"),
                Card::from("3s"),
                Card::from("4s"),
            ]),
            Card::from("5c"),
            Vec::from([HandScorings::FourCardFlush(HashSet::from_iter([
                Card::from("As"),
                Card::from("2s"),
                Card::from("3s"),
                Card::from("4s"),
            ]))]),
        )]);
        for (description, hand, up_card, expected) in tests {
            assert!(!contains(&hand, &up_card, &expected,), "{}", description,);
        }
    }

    #[test]
    fn five_card_flush() {
        let tests: Vec<(&str, HashSet<Card>, Card, Vec<HandScorings>, Vec<HandScorings>)> =
            Vec::from([(
                "Five card flush",
                HashSet::from_iter(vec![
                    Card::from("As"),
                    Card::from("2s"),
                    Card::from("3s"),
                    Card::from("4s"),
                ]),
                Card::from("5s"),
                Vec::from([HandScorings::FiveCardFlush(HashSet::from_iter([
                    Card::from("As"),
                    Card::from("2s"),
                    Card::from("3s"),
                    Card::from("4s"),
                    Card::from("5s"),
                ]))]),
                Vec::from([
                    HandScorings::FourCardFlush(HashSet::from_iter([
                        Card::from("As"),
                        Card::from("2s"),
                        Card::from("3s"),
                        Card::from("4s"),
                    ])),
                    HandScorings::FourCardFlush(HashSet::from_iter([
                        Card::from("2s"),
                        Card::from("3s"),
                        Card::from("4s"),
                        Card::from("5s"),
                    ])),
                ]),
            )]);
        for (description, hand, up_card, expected, unexpecteds) in tests {
            assert!(contains(&hand, &up_card, &expected,), "{}", description,);
            for unexpected in unexpecteds {
                assert!(
                    !contains(&hand, &up_card, &Vec::from([unexpected.clone()])),
                    "{} {:?}",
                    description,
                    unexpected
                );
            }
        }
    }
}

#[cfg(test)]
mod pegging_scoring {
    use super::*;
    use crate::deck::Card;

    #[test]
    fn test_score_pegging() {
        let tests: Vec<(&str, Vec<Card>, Vec<PeggingScorings>)> = Vec::from([
            ("No cards played", Vec::new(), Vec::new()),
            (
                "Non-scoring card played",
                Vec::from([Card::from("as")]),
                Vec::new(),
            ),
            (
                "Fifteen",
                Vec::from([Card::from("7s"), Card::from("8c")]),
                Vec::from([PeggingScorings::Fifteen]),
            ),
            (
                "Thirty One",
                Vec::from([
                    Card::from("Ts"),
                    Card::from("Tc"),
                    Card::from("Th"),
                    Card::from("Ah"),
                ]),
                Vec::from([PeggingScorings::ThirtyOne]),
            ),
            (
                "Pair",
                Vec::from([Card::from("as"), Card::from("ac")]),
                Vec::from([PeggingScorings::Pair(HashSet::from_iter(
                    [Card::from("ac"), Card::from("as")].iter().cloned(),
                ))]),
            ),
            (
                "Pair royale",
                Vec::from([Card::from("as"), Card::from("ac"), Card::from("ad")]),
                Vec::from([
                    PeggingScorings::Pair(HashSet::from_iter(
                        [Card::from("ac"), Card::from("as")].iter().cloned(),
                    )),
                    PeggingScorings::Pair(HashSet::from_iter(
                        [Card::from("ac"), Card::from("ad")].iter().cloned(),
                    )),
                    PeggingScorings::Pair(HashSet::from_iter(
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
                    PeggingScorings::Pair(HashSet::from_iter(
                        [Card::from("as"), Card::from("ac")].iter().cloned(),
                    )),
                    PeggingScorings::Pair(HashSet::from_iter(
                        [Card::from("as"), Card::from("ad")].iter().cloned(),
                    )),
                    PeggingScorings::Pair(HashSet::from_iter(
                        [Card::from("as"), Card::from("ah")].iter().cloned(),
                    )),
                    PeggingScorings::Pair(HashSet::from_iter(
                        [Card::from("ac"), Card::from("ad")].iter().cloned(),
                    )),
                    PeggingScorings::Pair(HashSet::from_iter(
                        [Card::from("ac"), Card::from("ah")].iter().cloned(),
                    )),
                    PeggingScorings::Pair(HashSet::from_iter(
                        [Card::from("ad"), Card::from("ah")].iter().cloned(),
                    )),
                ]),
            ),
            (
                "Run of three",
                Vec::from([Card::from("as"), Card::from("2s"), Card::from("3s")]),
                Vec::from([PeggingScorings::RunOfThree]),
            ),
            (
                "Run of four",
                Vec::from([
                    Card::from("as"),
                    Card::from("2s"),
                    Card::from("3s"),
                    Card::from("4s"),
                ]),
                Vec::from([PeggingScorings::RunOfFour]),
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
                Vec::from([PeggingScorings::RunOfFive]),
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
                Vec::from([PeggingScorings::RunOfSix]),
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
