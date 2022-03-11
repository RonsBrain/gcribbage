#include "simulation.h"
#include <gtk/gtk.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

enum GameState {
  STATE_CHOOSE_DEALER,
  STATE_CHOOSE_CRIB,
  STATE_ANNOUNCE_NIBS,
  STATE_PEGGING,
  STATE_COUNTING,
  STATE_WINNER,
  STATE_END
};

struct GameData {
  struct Card human_hand[6];
  struct Card cpu_hand[6];
  struct Card crib_hand[4];
  struct Card up_card;
  int cut_card_positions[2];
  int human_crib_choices[2];
  int scores[PLAYER_END];
  enum GameState state;
  enum PlayerType dealer;
};

struct Card possible_cards[52] = {};
struct Card CARD_NONE = {9, 0, 0};

/* Uses rejection sampling to return a random value between min and max
 * inclusive */
int get_random_number(int min, int max) {
  int limit = RAND_MAX - (RAND_MAX % (max - min));
  int r;
  while ((r = rand()) >= limit)
    ;
  return r % max + min;
}

void get_random_cards(int num_cards, struct Card *cards) {
  struct Card *current = cards;
  char chosen_cards[52] = {};
  int choice;
  for (int i = 0; i < num_cards; i++) {
    do {
      choice = get_random_number(0, 51);
    } while (chosen_cards[choice]);
    *current = possible_cards[choice];
    chosen_cards[choice] = 1;
    current++;
  }
}

struct GameData *game_data_create() {
  /* Not the best place to initialize random number seed. */
  srand(time(0));

  if (possible_cards[0].rank == 0) {
    for (int suit = 0; suit < 4; suit++) {
      for (int rank = 1; rank < 15; rank++) {
        possible_cards[suit * 13 + rank].suit = suit; 
        possible_cards[suit * 13 + rank].rank = rank; 
        possible_cards[suit * 13 + rank].value = rank < 10 ? rank : 10; 
      }
    }
  }
  /* 0 is a valid value for all initial values of GameData.
   * If this ever becomes not the case, then the allocated memory
   * should be initialized before returning it.
   */
  struct GameData *game_data =
      (struct GameData *)calloc(sizeof(struct GameData), 1);
  return game_data;
}

void game_data_destroy(struct GameData *game_data) {
  if (game_data) {
    free(game_data);
  }
}

void game_data_handle_state_not_implemented(struct GameData *game_data,
                                            int position) {}

void game_data_handle_state_choose_dealer(struct GameData *game_data,
                                          int human_choice_position) {
  if (human_choice_position != POSITION_NONE) {
    struct Card chosen[2] = {};
    while ((chosen[0].rank) == (chosen[1].rank)) {
      /* Let's not entertain tie cuts. Always make the cut have a winner. */
      get_random_cards(2, chosen);
    }
    game_data->human_hand[0] = chosen[0];
    game_data->cpu_hand[0] = chosen[1];
    game_data->cut_card_positions[0] = human_choice_position;
    game_data->cut_card_positions[1] = get_random_number(1, 13);
    game_data->dealer = chosen[0].rank < chosen[1].rank ? PLAYER_HUMAN : PLAYER_CPU;
  } else {
    game_data->dealer = PLAYER_NONE;
    if (game_data->cut_card_positions[0] != POSITION_NONE) {
      /* Cards already cut, move on to crib building. */
      game_data->state = STATE_CHOOSE_CRIB;
      struct Card cards[13];
      get_random_cards(13, cards);
      for (int i = 0; i < 6; i++) {
        game_data->human_hand[i] = cards[i];
        game_data->cpu_hand[i] = cards[i + 6];
      }
      game_data->up_card = cards[12];
    }
  }
}

void game_data_handle_state_choose_crib(struct GameData *game_data,
                                        int human_choice_position) {
  int ready_to_proceed = (game_data->human_crib_choices[0] != POSITION_NONE &&
                          game_data->human_crib_choices[1] != POSITION_NONE);

  if (human_choice_position != POSITION_NONE) {
    int removed = 0;
    for (int i = 0; i < 2; i++) {
      if (game_data->human_crib_choices[i] == human_choice_position) {
        game_data->human_crib_choices[i] = POSITION_NONE;
        removed = 1;
      }
    }
    if (!removed) {
      if (game_data->human_crib_choices[0] != POSITION_NONE &&
          game_data->human_crib_choices[1] != POSITION_NONE) {
        /* Player has chosen two cards already. Choosing another is illegal.
         * Do not advance the game state.
         */
        return;
      }
      if (game_data->human_crib_choices[0] == POSITION_NONE) {
        game_data->human_crib_choices[0] = human_choice_position;
      } else {
        game_data->human_crib_choices[1] = human_choice_position;
      }
    }
  } else {
    if (!ready_to_proceed) {
      for (int i = 0; i < 2; i++) {
        game_data->human_crib_choices[i] = 0;
      }
    } else {
      for (int i = 0; i < 2; i++) {
        game_data->crib_hand[i] =
            game_data->human_hand[game_data->human_crib_choices[i] - 1];
        game_data->human_hand[game_data->human_crib_choices[i] - 1] = CARD_NONE;
        /* TODO: Implement actual CPU AI instead of random crib choice */
        game_data->crib_hand[i + 2] = game_data->cpu_hand[i];
        game_data->cpu_hand[i] = CARD_NONE;
      }

      /* Move all cards in play to the beginning of the hand array */
      struct Card *card_dest, *card_temp, *card_current;
      struct Card temp[4];
      for (enum PlayerType player = PLAYER_HUMAN; player < PLAYER_END;
           player++) {
        card_temp = temp;
        if (player == PLAYER_HUMAN) {
          card_dest = card_current = game_data->human_hand;
        } else {
          card_dest = card_current = game_data->cpu_hand;
        }
        for (int i = 0; i < 6; i++) {
          if (card_current->rank) {
            *card_temp = *card_current;
            card_temp++;
          }
          card_current++;
        }
        for (int i = 0; i < 4; i++) {
          *card_dest = temp[i];
          card_dest++;
        }
      }
      if (game_data->up_card.rank == 11) {
        /* Dealer turned over a jack for two points. */
        game_data->scores[game_data->dealer] += 2;
        game_data->state = STATE_ANNOUNCE_NIBS;
      } else {
        game_data->state = STATE_PEGGING;
      }
    }
  }
}

void game_data_handle_state_announce_nibs(struct GameData *game_data,
                                          int human_choice_position) {
  game_data->state = STATE_PEGGING;
}

void (*state_handlers[STATE_END])(struct GameData *, int) = {
    &game_data_handle_state_choose_dealer, &game_data_handle_state_choose_crib,
    &game_data_handle_state_announce_nibs,
    &game_data_handle_state_not_implemented,
    &game_data_handle_state_not_implemented};

/* This advances the game state. GameData is opaque to the rest of the
 * program, so we can be sure that only this function is changing
 * game state
 */
void game_data_advance_game(struct GameData *game_data,
                            int human_choice_position) {
  state_handlers[game_data->state](game_data, human_choice_position);
}

/* Since GameData is opaque, we need to give information about what
 * scene should be rendered.
 *
 * Each scene type contains all the information necessary to render a scene.
 * The rendering widget needs to figure out how to do the rendering.
 */
void game_data_get_render_scene(struct GameData *game_data,
                                struct RenderScene *scene) {
  if (!game_data) {
    scene->type = BLANK_SCENE;
    return;
  }
  switch (game_data->state) {
  case STATE_CHOOSE_DEALER:
    scene->type = DECK_CUT_SCENE;
    scene->deck_cut_scene.human_card = game_data->human_hand[0];
    scene->deck_cut_scene.cpu_card = game_data->cpu_hand[0];
    scene->deck_cut_scene.chosen_slots[0] = game_data->cut_card_positions[0];
    scene->deck_cut_scene.chosen_slots[1] = game_data->cut_card_positions[1];
    scene->deck_cut_scene.first_dealer = game_data->dealer;
    break;
  case STATE_CHOOSE_CRIB:
    scene->type = CHOOSE_CRIB_SCENE;
    for (int i = 0; i < 6; i++) {
      scene->choose_crib_scene.human_cards[i] = game_data->human_hand[i];
    }
    scene->choose_crib_scene.human_crib_choices[0] =
        game_data->human_crib_choices[0];
    scene->choose_crib_scene.human_crib_choices[1] =
        game_data->human_crib_choices[1];
    scene->choose_crib_scene.ready_to_proceed =
        (game_data->human_crib_choices[0] != POSITION_NONE &&
         game_data->human_crib_choices[1] != POSITION_NONE);
    scene->choose_crib_scene.crib_player = game_data->dealer;
    for (enum PlayerType i = PLAYER_HUMAN; i < PLAYER_END; i++) {
      scene->choose_crib_scene.scores[i] = game_data->scores[i];
    }
    break;
  case STATE_ANNOUNCE_NIBS:
    scene->type = ANNOUNCE_NIBS_SCENE;
    for (int i = 0; i < 4; i++) {
      scene->announce_nibs_scene.human_cards[i] = game_data->human_hand[i];
      scene->announce_nibs_scene.up_card = game_data->up_card;
    }
    scene->announce_nibs_scene.scores[PLAYER_HUMAN] =
        game_data->scores[PLAYER_HUMAN];
    scene->announce_nibs_scene.scores[PLAYER_CPU] =
        game_data->scores[PLAYER_CPU];
    scene->announce_nibs_scene.dealer = game_data->dealer;
    break;
  default:
    scene->type = BLANK_SCENE;
  }
}
