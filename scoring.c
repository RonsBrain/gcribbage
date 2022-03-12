#include "scoring.h"

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

/* Start at the end of the card list. Look for the min and max rank in
 * the last three cards. If the min rank and max rank are two apart,
 * you have a run of three. Do this for four cards, five cards, etc.
 * until you max out the number of cards in the list.
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
  int max_cards = 1;
  while (!IS_SAME_CARD((*last_card), CARD_NONE)) {
    last_card++;
    max_cards++;
  }
  last_card--;
  if (last_card == cards || last_card == cards + 1) {
    return 0;
  }
  int run_length = 3, max_run_length = 0;

  while (run_length < max_cards) {
    int min = last_card->rank, max = min;
    current_card = last_card - 1;
    while (current_card > last_card - run_length && current_card >= cards) {
      if (current_card->rank < min) {
        min = current_card->rank;
      }

      if (current_card->rank > max) {
        max = current_card->rank;
      }
      current_card--;
    }
    if (max - min == run_length - 1) {
      max_run_length = run_length;
    }
    run_length++;
  }
  return max_run_length;
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
