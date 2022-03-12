#pragma once

struct Card {
  char suit;
  char rank;
  char value;
};

extern struct Card CARD_NONE;

#define IS_SAME_CARD(x, y) (x.suit == y.suit && x.rank == y.rank)
