#pragma once

#define POSITION_NONE 0
#define MAX_INSTRUCTIONS 32

struct GameData;

enum RenderType {
  BLANK_SCENE,
  DECK_CUT_SCENE,
  CHOOSE_CRIB_SCENE,
  ANNOUNCE_NIBS_SCENE,
};

enum PlayerType { PLAYER_NONE, PLAYER_HUMAN, PLAYER_CPU, PLAYER_END };

struct Card{
  char suit;
  char rank;
  char value;
};

extern struct Card CARD_NONE;

struct BlankScene {};

struct RenderDeckCutScene {
  struct Card human_card;
  struct Card cpu_card;
  int chosen_slots[2];
  enum PlayerType first_dealer;
};

struct ChooseCribScene {
  int ready_to_proceed;
  struct Card human_cards[6];
  int human_crib_choices[2];
  enum PlayerType crib_player;
  int scores[PLAYER_END];
};

struct AnnounceNibsScene {
  struct Card human_cards[4];
  struct Card up_card;
  int scores[PLAYER_END];
  enum PlayerType dealer;
};

struct RenderScene {
  enum RenderType type;
  union {
    struct BlankScene blank_scene;
    struct RenderDeckCutScene deck_cut_scene;
    struct ChooseCribScene choose_crib_scene;
    struct AnnounceNibsScene announce_nibs_scene;
  };
};

struct GameData *game_data_create();
void game_data_destroy(struct GameData *game_data);
void game_data_advance_game(struct GameData *game_data,
                            int human_choice_position);
void game_data_get_render_scene(struct GameData *game_data,
                                struct RenderScene *scene);
