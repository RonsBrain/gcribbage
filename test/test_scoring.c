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

int run_test_case(char *case_to_parse) {
  struct Card test_cards[9];
  struct Card *current_card = test_cards;
  int failure = 0;
  char *original_case = case_to_parse;
  enum ScoreType scores[24];
  enum ScoreType expected_scores[3];
  enum ScoreType *current_score = expected_scores;
  int parsing_cards = 1;
  int last_card = 0;

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
        current_card++;
        break;
      case '0':
        current_card->rank = 10;
        current_card->value = 10;
        current_card++;
        break;
      case 'J':
        current_card->rank = 11;
        current_card->value = 10;
        current_card++;
        break;
      case 'Q':
        current_card->rank = 12;
        current_card->value = 10;
        current_card++;
        break;
      case 'K':
        current_card->rank = 13;
        current_card->value = 10;
        current_card++;
        break;
      case 'L':
        last_card = 1;
        break;
      case ' ':
        parsing_cards = 0;
        break;
      default:
        break;
      }
    } else {
      *current_score = *case_to_parse - 'A';
      current_score++;
    }
    case_to_parse++;
  }
  *current_score = SCORE_DONE;
  *current_card = CARD_NONE;
  score_pegging(test_cards, scores, last_card);
  enum ScoreType *expected, *actual;
  expected = expected_scores;
  actual = scores;
  do {
    if (*actual != *expected) {
      printf("FAIL - %s wanted %s got %s \n", original_case,
             score_names[*expected], score_names[*actual]);
      failure = 1;
      break;
    }
    actual++;
    expected++;
  } while (*(actual - 1) != SCORE_DONE);

  return failure;
}

int main(int argc, char **argv) {
  int failures = 0;

  /* Test cases are defined by a series of card identifiers, a possible
   * last card identifier, and finally a serices of score type identifiers.
   *
   * The card identifiers are basically the rank of the card to use,
   * with '0' representing a ten, 'J', 'Q', 'K' representing jack,
   * queen, and king respectively.
   *
   * The last card identifier is 'L'.
   *
   * The score identifiers are the enum offset for the ScoreType that is
   * expected, with 'A' being SCORE_NONE, 'B' being SCORE_FIFTEEN, etc.
   *
   * The last entry is a terminal entry, which should end the testing.
   */
  char *test_cases[] = {"1",           "123 H",       "1234 I",      "12345 BJ",
                        "123456 K",    "1234567 L",   "78 B",        "96 B",
                        "05 B",        "J5 B",        "Q5 B",        "K5 B",
                        "041 B",       "J41 B",       "Q41 B",       "K41 B",
                        "032 B",       "J32 B",       "Q32 B",       "K32 B",
                        "44322 BE",    "KQ0L D",      "1245 ",       "JJQK H",
                        "12334567 CJ", "14253637 CK", "74253631 CK", "1JQK CH",
                        "11 E",        "111 F",       "1111 G",      "\0"};

  int current_case = 0;
  char *case_to_parse = test_cases[current_case];

  do {
    failures += run_test_case(case_to_parse);
    current_case++;
    case_to_parse = test_cases[current_case];
  } while (*case_to_parse != '\0');
  return failures != 0;
}
