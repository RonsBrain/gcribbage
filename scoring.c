#include "scoring.h"
#include <stdio.h>

int is_fifteen(struct Card *cards) {
  int total = 0;
  while (!IS_SAME_CARD((*cards), CARD_NONE)) {
    total += cards->value;
    cards++;
  }

  return total == 15;
}

int is_thirty_one(struct Card *cards) {
  int total = 0;
  while (!IS_SAME_CARD((*cards), CARD_NONE)) {
    total += cards->value;
    cards++;
  }

  return total == 31;
}

/* Start at the end of the card list and see how many cards
 * in a row have the same rank.
 */
int find_pair_length(struct Card *cards) {
  struct Card *current_card = cards;
  while (!IS_SAME_CARD((*current_card), CARD_NONE)) {
    current_card++;
  }
  current_card--;
  if (current_card == cards) {
    return 0;
  }
  int last_rank = current_card->rank;
  current_card--;

  int pairs = 0;

  while (current_card >= cards) {
    if (current_card->rank == last_rank) {
      pairs++;
    } else {
      break;
    }
    last_rank = current_card->rank;
    current_card--;
  }

  return pairs;
}

/* Finding a run of a given length is essentially seeing if the min
 * rank and the max rank differ by the desired run length minus one.
 * We need to keep track of what ranks we have seen, however, because
 * looking for a run of 4 in 9696 would be a false positive. If we
 * see the same rank twice, then the set of cards cannot possibly
 * be a run of the desired length.
 */
int is_run(struct Card *cards, int run_length) {
  int min_rank = cards->rank, max_rank = cards->rank;
  int seen_ranks[13] = {};
  while (!IS_SAME_CARD((*cards), CARD_NONE)) {
    if (seen_ranks[(int)cards->rank - 1]) {
      /* We've already seen this rank, this can't possibly be
       * a run of the length we're interested in. This prevents
       * erroneous runs like 9696 or 54341
       */
      return 0;
    }
    seen_ranks[(int)cards->rank - 1] = 1;
    if (cards->rank < min_rank) {
      min_rank = cards->rank;
    }
    if (cards->rank > max_rank) {
      max_rank = cards->rank;
    }
    cards++;
  }
  return (max_rank - min_rank) == (run_length - 1);
}

/* Start at the end of the card list. Look to see if this is a run of seven.
 * Do this for six cards and a run of six, five cards and a run of five, etc.
 * until you try a run of three.
 *
 * The maximum run that can occur during pegging is a run of seven.
 * This would be Ace through seven which is a total pegging count of
 * 28, making an eight play impossible. Two through 8 is 35 points,
 * so that is also impossible.
 *
 * It's important that the difference between the min and max is
 * one less the run length the algorithm is currently looking at.
 * This prevents situations where the last four cards are
 * ace, ace, two, three and the algorithm incorrectly identifies
 * this as a run of four.
 */
int find_run_length(struct Card *cards) {
  struct Card *last_card = cards;
  struct Card *current_card;
  int run_length = 0;
  while (!IS_SAME_CARD((*last_card), CARD_NONE)) {
    last_card++;
    run_length++;
  }
  if (run_length < 3) {
    /* Not enough cards for a run. */
    return 0;
  }
  if (run_length > 7) {
    /* Can't have more than a run of 7. */
    run_length = 7;
  }
  /* Above increment puts last_card past the last card, so rewind the last
   * increment. */
  last_card--;

  struct Card to_score[8] = {};
  while (run_length > 2) {
    current_card = last_card;
    for (int i = 0; i < run_length; i++) {
      to_score[i] = *current_card;
      current_card--;
    }
    to_score[run_length] = CARD_NONE;
    if (is_run(to_score, run_length)) {
      return run_length;
    } else {
      run_length--;
    }
  }
  return 0;
}

enum ScoreType *add_score(enum ScoreType *current, enum ScoreType score) {
  *current = score;
  current++;
  return current;
}

void score_pegging(struct Card *cards, enum ScoreType scores[24],
                   int is_last_card) {
  enum ScoreType *current_score = scores;
  if (is_fifteen(cards)) {
    current_score = add_score(current_score, SCORE_FIFTEEN);
  }

  if (is_thirty_one(cards)) {
    current_score = add_score(current_score, SCORE_THIRTY_ONE);
  }

  int pair_length = find_pair_length(cards);
  if (pair_length > 0) {
    /* Add the right pair type based on number of pairs.
     * This assumes the pair types are next to each other
     * in the enum.
     */
    current_score = add_score(current_score, SCORE_PAIR + pair_length - 1);
  }

  int run_length = find_run_length(cards);
  if (run_length > 0) {
    /* Add the right run type based on length of run.
     * This assumes the run types are next to each other
     * in the enum.
     */
    current_score =
        add_score(current_score, SCORE_RUN_OF_THREE + run_length - 3);
  }

  if (is_last_card && !is_thirty_one(cards)) {
    current_score = add_score(current_score, SCORE_LAST_CARD);
  }

  add_score(current_score, SCORE_DONE);
}

enum ScoreType *score_fifteens(struct Card *cards,
                               enum ScoreType *current_score) {
  /* This is a list of all combinations of card positions that result in
   * two or more cards. Using this list instead of computing all relevant
   * combinations at runtime is, in my opinion, easier to read.
   */
  int check_positions[26][5] = {
      {3, 4, -1, -1, -1}, {2, 4, -1, -1, -1}, {2, 3, -1, -1, -1},
      {2, 3, 4, -1, -1},  {1, 4, -1, -1, -1}, {1, 3, -1, -1, -1},
      {1, 3, 4, -1, -1},  {1, 2, -1, -1, -1}, {1, 2, 4, -1, -1},
      {1, 2, 3, -1, -1},  {1, 2, 3, 4, -1},   {0, 4, -1, -1, -1},
      {0, 3, -1, -1, -1}, {0, 3, 4, -1, -1},  {0, 2, -1, -1, -1},
      {0, 2, 4, -1, -1},  {0, 2, 3, -1, -1},  {0, 2, 3, 4, -1},
      {0, 1, -1, -1, -1}, {0, 1, 4, -1, -1},  {0, 1, 3, -1, -1},
      {0, 1, 3, 4, -1},   {0, 1, 2, -1, -1},  {0, 1, 2, 4, -1},
      {0, 1, 2, 3, -1},   {0, 1, 2, 3, 4}};

  struct Card to_score[6] = {};
  to_score[5] = CARD_NONE;
  /* Go over each combo */
  for (int i = 0; i < 26; i++) {
    struct Card *current = to_score;
    /* Populate the cards to score from the current combo positions */
    for (int j = 0; j < 5; j++) {
      int position = check_positions[i][j];
      if (position > -1) {
        *current = cards[position];
      } else {
        *current = CARD_NONE;
      }
      current++;
    }
    if (is_fifteen(to_score)) {
      current_score = add_score(current_score, SCORE_FIFTEEN);
    }
  }
  return current_score;
}

enum ScoreType *score_pairs(struct Card *cards, enum ScoreType *current_score) {
  /* Keep track of how many of which rank we have seen. */
  int seen_ranks[13] = {};
  while (!IS_SAME_CARD((*cards), CARD_NONE)) {
    seen_ranks[cards->rank - 1] += 1;
    cards++;
  }

  for (int i = 0; i < 13; i++) {
    if (seen_ranks[i] > 1) {
      /* There is at least a pair. Add a score related to how many of
       * this rank was seen.
       */
      current_score = add_score(current_score, SCORE_PAIR + seen_ranks[i] - 2);
    }
  }
  return current_score;
}

enum ScoreType *score_runs(struct Card *cards, enum ScoreType *current_score) {
  int four_card_combos[5][4] = {
      {1, 2, 3, 4}, {0, 2, 3, 4}, {0, 1, 3, 4}, {0, 1, 2, 4}, {0, 1, 2, 3}};

  int three_card_combos[10][3] = {{2, 3, 4}, {1, 3, 4}, {1, 2, 4}, {1, 2, 3},
                                  {0, 3, 4}, {0, 2, 4}, {0, 2, 3}, {0, 1, 4},
                                  {0, 1, 3}, {0, 1, 2}};

  if (is_run(cards, 5)) {
    return add_score(current_score, SCORE_RUN_OF_FIVE);
  }

  int scored = 0;
  struct Card to_score[6] = {};
  struct Card *current;
  for (int i = 0; i < 5; i++) {
    current = to_score;
    for (int j = 0; j < 4; j++) {
      int position = four_card_combos[i][j];
      *current = cards[position];
      current++;
    }
    *current = CARD_NONE;
    if (is_run(to_score, 4)) {
      current_score = add_score(current_score, SCORE_RUN_OF_FOUR);
      scored = 1;
    }
  }

  if (scored) {
    return current_score;
  }

  for (int i = 0; i < 10; i++) {
    current = to_score;
    for (int j = 0; j < 3; j++) {
      int position = three_card_combos[i][j];
      *current = cards[position];
      current++;
    }
    *current = CARD_NONE;
    if (is_run(to_score, 3)) {
      current_score = add_score(current_score, SCORE_RUN_OF_THREE);
    }
  }
  return current_score;
}

enum ScoreType *score_flush(struct Card *cards, enum ScoreType *current_score,
                            int is_crib) {
  int seen_suits[4] = {};
  while (!IS_SAME_CARD((*cards), CARD_NONE)) {
    seen_suits[(int)cards->suit]++;
    cards++;
  }
  for (int i = 0; i < 4; i++) {
    if ((seen_suits[i] == 4 && !is_crib) || seen_suits[i] == 5) {
      current_score = add_score(current_score, SCORE_FLUSH + seen_suits[i] - 4);
    }
  }

  return current_score;
}

enum ScoreType *score_nobs(struct Card *cards, enum ScoreType *current_score) {
  struct Card *up_card = &cards[4];
  while (cards < up_card) {
    if ((cards->rank == 11) && (cards->suit == up_card->suit)) {
      /* Card is a Jack of the same suit as the up card. */
      return add_score(current_score, SCORE_NOBS);
    }
    cards++;
  }
  return current_score;
}

void score_counting(struct Card *cards, enum ScoreType scores[24],
                    int is_crib) {
  enum ScoreType *current_score = scores;

  current_score = score_fifteens(cards, current_score);
  current_score = score_pairs(cards, current_score);
  current_score = score_runs(cards, current_score);
  current_score = score_flush(cards, current_score, is_crib);
  current_score = score_nobs(cards, current_score);
  add_score(current_score, SCORE_DONE);
}
