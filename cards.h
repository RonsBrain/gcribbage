#pragma once

struct Card {
  char suit;
  char rank;
  char value;
};

extern struct Card CARD_NONE;

#define IS_CARD(x) (!(x.suit == 9 || x.rank == 0))
