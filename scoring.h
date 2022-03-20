#pragma once
#include "simulation.h"

enum ScoreType {
  SCORE_NONE,
  SCORE_FIFTEEN,
  SCORE_THIRTY_ONE,
  SCORE_LAST_CARD,
  SCORE_PAIR,
  SCORE_PAIR_ROYALE,
  SCORE_DOUBLE_PAIR_ROYALE,
  SCORE_RUN_OF_THREE,
  SCORE_RUN_OF_FOUR,
  SCORE_RUN_OF_FIVE,
  SCORE_RUN_OF_SIX,
  SCORE_RUN_OF_SEVEN,
  SCORE_FLUSH,
  SCORE_FIVE_FLUSH,
  SCORE_NOBS,
  SCORE_DONE
};

void score_pegging(struct Card *cards, enum ScoreType scores[24],
                   int is_last_card);
void score_counting(struct Card *cards, enum ScoreType scores[24],
                    int is_pegging);
