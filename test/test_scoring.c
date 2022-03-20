#include "../cards.h"
#include "../scoring.h"
#include <stdio.h>

char *score_names[SCORE_DONE + 1] = {"NONE",
                                     "FIFTEEN",
                                     "THIRTY ONE",
                                     "LAST_CARD",
                                     "PAIR",
                                     "PAIR ROYALE",
                                     "DOUBLE PAIR ROYALE",
                                     "RUN OF THREE",
                                     "RUN OF FOUR",
                                     "RUN OF FIVE",
                                     "RUN OF SIX",
                                     "RUN OF SEVEN",
                                     "FLUSH",
                                     "FIVE FLUSH",
                                     "NOBS",
                                     "DONE"};

void parse_test_case(char *case_to_parse, struct Card *test_cards,
                     enum ScoreType *expected_scores, int *special_flag) {
  struct Card *current_card = test_cards;
  enum ScoreType *current_score = expected_scores;
  int parsing_cards = 1;
  *special_flag = 0;
  int suit = 0;
  int flush_type = 0;
  int nobs_suit = -1;
  while (*case_to_parse != '\0') {
    if (parsing_cards) {
      switch (*case_to_parse) {
      case '1':
      case '2':
      case '3':
      case '4':
      case '5':
      case '6':
      case '7':
      case '8':
      case '9':
        current_card->rank = *case_to_parse - '9' + 9;
        current_card->value = *case_to_parse - '9' + 9;
        current_card->suit = suit % 4;
        current_card++;
        break;
      case '0':
        current_card->rank = 10;
        current_card->value = 10;
        current_card->suit = suit % 4;
        current_card++;
        break;
      case 'J':
        current_card->rank = 11;
        current_card->value = 10;
        current_card->suit = suit % 4;
        current_card++;
        break;
      case 'Q':
        current_card->rank = 12;
        current_card->value = 10;
        current_card->suit = suit % 4;
        current_card++;
        break;
      case 'K':
        current_card->rank = 13;
        current_card->value = 10;
        current_card->suit = suit % 4;
        current_card++;
        break;
      case 'f':
        flush_type = 1;
        break;
      case 'F':
        flush_type = 2;
        break;
      case 'L':
      case 'C':
        *special_flag = 1;
        break;
      case ' ':
        parsing_cards = 0;
        break;
      case 'N':
        current_card->rank = 11;
        current_card->value = 10;
        current_card->suit = suit % 4;
        current_card++;
        nobs_suit = suit % 4;
        break;
      default:
        break;
      }
    } else {
      *current_score = *case_to_parse - 'A';
      current_score++;
    }
    suit++;
    case_to_parse++;
  }
  *current_score = SCORE_DONE;
  *current_card = CARD_NONE;
  if (flush_type > 0) {
    current_card = test_cards;
    while (!IS_SAME_CARD((*current_card), CARD_NONE)) {
      current_card->suit = 0;
      current_card++;
    }
    if (flush_type == 1) {
      (current_card - 1)->suit = 1;
    }
  } else if (nobs_suit > -1) {
    (current_card - 1)->suit = nobs_suit;
  }
}

int verify_results(char *test_case, enum ScoreType *expected,
                   enum ScoreType *actual) {
  int failure = 0;
  do {
    if (*actual != *expected) {
      printf("FAIL - %s wanted %s got %s \n", test_case, score_names[*expected],
             score_names[*actual]);
      failure = 1;
      break;
    }
    actual++;
    expected++;
  } while (*(actual - 1) != SCORE_DONE);
  return failure;
}

int test_pegging(char *test_cases[]) {
  char **current_case;
  current_case = test_cases;
  int failures = 0;
  struct Card test_cards[9];
  enum ScoreType scores[24] = {};
  enum ScoreType expected_scores[24] = {};
  int last_card;

  while (**current_case != '\0') {
    parse_test_case(*current_case, test_cards, expected_scores, &last_card);
    score_pegging(test_cards, scores, last_card);
    failures += verify_results(*current_case, expected_scores, scores);
    current_case++;
  }
  return failures;
}

int test_counting(char *test_cases[]) {
  char **current_case;
  current_case = test_cases;
  int failures = 0;
  struct Card test_cards[6] = {};
  enum ScoreType scores[24] = {};
  enum ScoreType expected_scores[24] = {};
  int is_pegging;

  test_cards[5] = CARD_NONE;

  while (**current_case != '\0') {
    parse_test_case(*current_case, test_cards, expected_scores, &is_pegging);
    score_counting(test_cards, scores, is_pegging);
    failures += verify_results(*current_case, expected_scores, scores);
    current_case++;
  };
  return failures;
}

int main(int argc, char **argv) {
  int failures = 0;

  /* Test cases are defined by a series of card identifiers, a possible
   * last card identifier, and finally a serices of score type identifiers.
   *
   * The card identifiers are basically the rank of the card to use,
   * with '0' representing a ten, 'J', 'Q', 'K' representing jack,
   * queen, and king respectively. If the rank is 'N' then this is a
   * jack that is the same suit as the up card.
   *
   *
   * The last card identifier is 'L'.
   *
   * The crib hand identifier is 'C'.
   *
   * If the test case starts with an 'f', then this should be a four card flush.
   * If it starts with an 'F', then this should be a five card flush.
   *
   * The score identifiers are the enum offset for the ScoreType that is
   * expected, with 'A' being SCORE_NONE, 'B' being SCORE_FIFTEEN, etc.
   *
   * The last entry is a terminal entry, which should end the testing.
   */
  char *pegging_test_cases[] = {
      "1",          "123 H",      "1234 I",  "12345 BJ", "123456 K",
      "1234567 L",  "78 B",       "96 B",    "05 B",     "J5 B",
      "Q5 B",       "K5 B",       "041 B",   "J41 B",    "Q41 B",
      "K41 B",      "032 B",      "J32 B",   "Q32 B",    "K32 B",
      "44322 BE",   "KQ0L D",     "1245 ",   "JJQK H",   "12334567 CJ",
      "14253637 C", "74253631 C", "1JQK CH", "11 E",     "111 F",
      "1111 G",     "9696",       "\0"};

  char *counting_test_cases[] = {"96961 BBBBEE",
                                 "391K7",
                                 "50297 B",
                                 "2222K G",
                                 "222KQ F",
                                 "12345 BJ",
                                 "54341 EHH",
                                 "45656 BBBBEEHHHH",
                                 "555J5 BBBBBBBBG",
                                 "fJ13K9 M",
                                 "FQ13K9 N",
                                 "F45678 BBJN",
                                 "31NK9 O",
                                 "555N5 BBBBBBBBGO",
                                 "fQ13K9C",
                                 "FQ13K9C N",
                                 "\0"};

  failures += test_pegging(pegging_test_cases);
  failures += test_counting(counting_test_cases);
  return failures != 0;
}
