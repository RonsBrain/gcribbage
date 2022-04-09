#include "simulation.h"
#include "scoring.h"
#include <gtk/gtk.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

enum GameState {
  STATE_CHOOSE_DEALER,
  STATE_ANNOUNCE_DEALER,
  STATE_CHOOSE_CRIB,
  STATE_ANNOUNCE_NIBS,
  STATE_PEGGING,
  STATE_ANNOUNCE_LAST_CARD,
  STATE_ANNOUNCE_THIRTY_ONE,
  STATE_COUNTING,
  STATE_WINNER,
  STATE_END
};

char *game_state_names[STATE_END] = {
    "CHOOSE_DEALER",       "ANNOUNCE_DEALER", "CHOOSE_CRIB",
    "ANNOUNCE_NIBS",       "PEGGING",         "ANNOUNCE_LAST_CARD",
    "ANNOUNCE_THIRTY_ONE", "COUNTING",        "WINNER",
};

char *advance_result_names[] = {
    "CONTINUE",
    "WAIT_FOR_USER",
};

char *player_type_names[PLAYER_END] = {"NONE", "HUMAN", "CPU"};

struct GameData {
  struct Card player_hands[PLAYER_END][6];
  struct Card original_hands[PLAYER_END][4];
  struct Card crib_hand[4];
  struct Card up_card;
  int cut_card_positions[PLAYER_END];
  int human_crib_choices[2];
  int scores[PLAYER_END];
  enum GameState state;
  enum PlayerType dealer;
  struct Card played_cards[8];
  struct Card *current_played_position;
  enum PlayerType current_player;
  int called_go[PLAYER_END];
  int pegging_count;
  enum PlayerType last_card_player;
  int remaining_cards[PLAYER_END];
  enum ScoreType score_list[24];
};

int SCORE_VALUES[SCORE_DONE] = {
    0,  // SCORE_NONE
    2,  // SCORE_FIFTEEN
    2,  // SCORE_THIRTY_ONE
    1,  // SCORE_LAST_CARD
    2,  // SCORE_PAIR
    6,  // SCORE_PAIR_ROYALE
    12, // SCORE_DOUBLE_PAIR_ROYALE
    3,  // SCORE_RUN_OF_THREE
    4,  // SCORE_RUN_OF_FOUR
    5,  // SCORE_RUN_OF_FIVE
    6,  // SCORE_RUN_OF_SIX
    7,  // SCORE_RUN_OF_SEVEN
    4,  // SCORE_FLUSH
    5,  // SCORE_FIVE_FLUSH
    1,  // SCORE_NOBS
};

struct Card possible_cards[52] = {
    {0, 1, 1},   {0, 2, 2},   {0, 3, 3},   {0, 4, 4},   {0, 5, 5},
    {0, 6, 6},   {0, 7, 7},   {0, 8, 8},   {0, 9, 9},   {0, 10, 10},
    {0, 11, 10}, {0, 12, 10}, {0, 13, 10}, {1, 1, 1},   {1, 2, 2},
    {1, 3, 3},   {1, 4, 4},   {1, 5, 5},   {1, 6, 6},   {1, 7, 7},
    {1, 8, 8},   {1, 9, 9},   {1, 10, 10}, {1, 11, 10}, {1, 12, 10},
    {1, 13, 10}, {2, 1, 1},   {2, 2, 2},   {2, 3, 3},   {2, 4, 4},
    {2, 5, 5},   {2, 6, 6},   {2, 7, 7},   {2, 8, 8},   {2, 9, 9},
    {2, 10, 10}, {2, 11, 10}, {2, 12, 10}, {2, 13, 10}, {3, 1, 1},
    {3, 2, 2},   {3, 3, 3},   {3, 4, 4},   {3, 5, 5},   {3, 6, 6},
    {3, 7, 7},   {3, 8, 8},   {3, 9, 9},   {3, 10, 10}, {3, 11, 10},
    {3, 12, 10}, {3, 13, 10}};

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

int card_comparator(const void *vleft, const void *vright) {
  const struct Card *left, *right;
  left = (const struct Card *)vleft;
  right = (const struct Card *)vright;
  int left_rank = left->rank, right_rank = right->rank;

  if (!IS_CARD((*left))) {
    left_rank = 999;
  }
  if (!IS_CARD((*right))) {
    right_rank = 999;
  }

  return left_rank - right_rank;
}

int has_valid_play(struct Card *cards, int score) {
  for (int i = 0; i < 4; i++) {
    if (IS_CARD(cards[i]) && cards[i].value + score <= 31) {
      return 1;
    }
  }
  return 0;
}

enum PlayerType get_next_player(enum PlayerType current) {
  if (current == PLAYER_HUMAN) {
    return PLAYER_CPU;
  } else if (current == PLAYER_CPU) {
    return PLAYER_HUMAN;
  } else {
    return PLAYER_NONE;
  }
}

void game_data_transition_to_choose_dealer(struct GameData *game_data) {
  game_data->dealer = PLAYER_NONE;
  game_data->up_card = CARD_NONE;
  game_data->current_player = PLAYER_HUMAN;

  for (int i = 0; i < 6; i++) {
    game_data->player_hands[PLAYER_HUMAN][i] = CARD_NONE;
    game_data->player_hands[PLAYER_CPU][i] = CARD_NONE;
    if (i < 4) {
      game_data->crib_hand[i] = CARD_NONE;
    }

    if (i < 2) {
      game_data->human_crib_choices[i] = POSITION_NONE;
    }

    if (i < PLAYER_END) {
      game_data->cut_card_positions[i] = POSITION_NONE;
      game_data->scores[i] = 0;
    }
  }
};

void game_data_transition_to_announce_dealer(struct GameData *game_data) {
  if (game_data->player_hands[PLAYER_HUMAN][0].rank <
      game_data->player_hands[PLAYER_CPU][0].rank) {
    game_data->dealer = PLAYER_HUMAN;
  } else {
    game_data->dealer = PLAYER_CPU;
  }
  game_data->current_player = PLAYER_NONE;
}

void game_data_transition_to_choose_crib(struct GameData *game_data) {
  struct Card cards[13];
  get_random_cards(13, cards);
  for (int i = 0; i < 6; i++) {
    game_data->player_hands[PLAYER_HUMAN][i] = cards[i];
    game_data->player_hands[PLAYER_CPU][i] = cards[i + 6];
  }
  qsort(game_data->player_hands[PLAYER_HUMAN], 6, sizeof(struct Card),
        &card_comparator);
  qsort(game_data->player_hands[PLAYER_CPU], 6, sizeof(struct Card),
        &card_comparator);
  game_data->up_card = cards[12];
  for (int i = 0; i < 2; i++) {
    game_data->human_crib_choices[i] = 0;
  }
}

void game_data_transition_to_announce_nibs(struct GameData *game_data) {
  game_data->scores[game_data->dealer] += 2;
}

void game_data_transition_to_pegging(struct GameData *game_data) {
  game_data->current_played_position = game_data->played_cards;
  game_data->pegging_count = 0;
  *game_data->played_cards = CARD_NONE;
  game_data->called_go[PLAYER_HUMAN] = 0;
  game_data->called_go[PLAYER_CPU] = 0;
  if (game_data->current_player == PLAYER_NONE) {
    /* Haven't actually started pegging yet, so deal cards and set things up. */
    game_data->current_player = get_next_player(game_data->dealer);
    game_data->remaining_cards[PLAYER_CPU] = 4;
    game_data->remaining_cards[PLAYER_HUMAN] = 4;
    for (int i = 0; i < 2; i++) {
      game_data->crib_hand[i] =
          game_data->player_hands[PLAYER_HUMAN]
                                 [game_data->human_crib_choices[i] - 1];
      game_data
          ->player_hands[PLAYER_HUMAN][game_data->human_crib_choices[i] - 1] =
          CARD_NONE;
      /* TODO: Implement actual CPU AI instead of random crib choice */
      game_data->crib_hand[i + 2] = game_data->player_hands[PLAYER_CPU][i];
      game_data->player_hands[PLAYER_CPU][i] = CARD_NONE;
    }
    /* Re-sort the cards to get CARD_NONE to the right */
    qsort(game_data->player_hands[PLAYER_HUMAN], 6, sizeof(struct Card),
          &card_comparator);
    qsort(game_data->player_hands[PLAYER_CPU], 6, sizeof(struct Card),
          &card_comparator);
    qsort(game_data->crib_hand, 4, sizeof(struct Card), &card_comparator);
    /* Keep track of the original hands for the count */
    for (int i = 0; i < 4; i++) {
      for (int player = 0; player < PLAYER_END; player++) {
        game_data->original_hands[player][i] =
            game_data->player_hands[player][i];
      }
    }
  } else {
    // game_data->current_player = get_next_player(game_data->current_player);
  }
}

void game_data_transition_to_announce_last_card(struct GameData *game_data) {
  game_data->score_list[0] = SCORE_LAST_CARD;
  game_data->score_list[1] = SCORE_DONE;
  game_data->scores[game_data->last_card_player] += 1;
}

void game_data_transition_to_announce_thirty_one(struct GameData *game_data) {
  game_data->current_player = get_next_player(game_data->current_player);
}

void game_data_transition_to_counting(struct GameData *game_data) {}

void (*transition_handlers[STATE_END])(struct GameData *) = {
    &game_data_transition_to_choose_dealer,
    &game_data_transition_to_announce_dealer,
    &game_data_transition_to_choose_crib,
    &game_data_transition_to_announce_nibs,
    &game_data_transition_to_pegging,
    &game_data_transition_to_announce_last_card,
    &game_data_transition_to_announce_thirty_one,
    &game_data_transition_to_counting};

void game_data_transition(struct GameData *game_data, enum GameState state) {
  g_log(G_LOG_DOMAIN, G_LOG_LEVEL_DEBUG, "Transitioning from %s to %s",
        game_state_names[game_data->state], game_state_names[state]);
  game_data->state = state;
  transition_handlers[state](game_data);
}

enum GameAdvanceResult
game_data_handle_state_choose_dealer(struct GameData *game_data,
                                     int human_choice_position) {
  switch (game_data->current_player) {
  case PLAYER_HUMAN:
    if (human_choice_position == POSITION_NONE) {
      /* We need to know what card the human choose. */
      return ADVANCE_RESULT_WAIT_FOR_USER;
    }
    /* Human chose a card. Pick a random one and give it to them.
     * Note that the human's selection doesn't matter.
     */
    get_random_cards(1, &game_data->player_hands[PLAYER_HUMAN][0]);
    game_data->cut_card_positions[0] = human_choice_position;
    game_data->current_player = PLAYER_CPU;
    return ADVANCE_RESULT_CONTINUE;
    break;
  case PLAYER_CPU:
    /* Simulator wants to choose a card now. Let's not choose the same rank as
     * the human. That way we don't have to write tiebreaker logic and scenes.
     */
    do {
      get_random_cards(1, &game_data->player_hands[PLAYER_CPU][0]);
    } while (game_data->player_hands[PLAYER_CPU][0].rank ==
             game_data->player_hands[PLAYER_HUMAN][0].rank);
    do {
      game_data->cut_card_positions[1] = get_random_number(1, 13);
    } while (game_data->cut_card_positions[1] ==
             game_data->cut_card_positions[0]);

    game_data_transition(game_data, STATE_ANNOUNCE_DEALER);
    return ADVANCE_RESULT_WAIT_FOR_USER;
    break;
  default:
    g_log(G_LOG_DOMAIN, G_LOG_LEVEL_CRITICAL,
          "Choose dealer state doesn't know who's turn it is");
    abort();
    return ADVANCE_RESULT_WAIT_FOR_USER;
  }
}

enum GameAdvanceResult
game_data_handle_state_announce_dealer(struct GameData *game_data,
                                       int human_choice_position) {
  game_data_transition(game_data, STATE_CHOOSE_CRIB);
  return ADVANCE_RESULT_WAIT_FOR_USER;
}

enum GameAdvanceResult
game_data_handle_state_choose_crib(struct GameData *game_data,
                                   int human_choice_position) {
  int ready_to_proceed = (game_data->human_crib_choices[0] != POSITION_NONE &&
                          game_data->human_crib_choices[1] != POSITION_NONE);

  if (ready_to_proceed && human_choice_position == POSITION_NONE) {
    /* Cards selected and the player is ready. */
    if (game_data->up_card.rank == 11) {
      /* Dealer turned over a jack for nibs. */
      game_data_transition(game_data, STATE_ANNOUNCE_NIBS);
      return ADVANCE_RESULT_WAIT_FOR_USER;
    } else {
      game_data_transition(game_data, STATE_PEGGING);
      return ADVANCE_RESULT_CONTINUE;
    }
  } else {
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
        return ADVANCE_RESULT_WAIT_FOR_USER;
      }
      if (game_data->human_crib_choices[0] == POSITION_NONE) {
        game_data->human_crib_choices[0] = human_choice_position;
      } else {
        game_data->human_crib_choices[1] = human_choice_position;
      }
    }
    return ADVANCE_RESULT_WAIT_FOR_USER;
  }
}

enum GameAdvanceResult
game_data_handle_state_announce_nibs(struct GameData *game_data,
                                     int human_choice_position) {
  game_data_transition(game_data, STATE_PEGGING);
  return ADVANCE_RESULT_CONTINUE;
}

enum GameAdvanceResult
game_data_handle_state_pegging(struct GameData *game_data,
                               int human_choice_position) {
  if (game_data->called_go[PLAYER_HUMAN] && game_data->called_go[PLAYER_CPU]) {
    game_data_transition(game_data, STATE_ANNOUNCE_LAST_CARD);
    return ADVANCE_RESULT_WAIT_FOR_USER;
  }

  struct Card played_card = CARD_NONE;
  if (game_data->pegging_count == 31) {
    /* Reset everything for the next round of pegging. */
    game_data_transition(game_data, STATE_ANNOUNCE_THIRTY_ONE);
  } else {
    if (game_data->remaining_cards[game_data->current_player] == 0 ||
        !has_valid_play(game_data->player_hands[game_data->current_player],
                        game_data->pegging_count)) {
      /* Player must call "go" */
      game_data->called_go[game_data->current_player] = 1;
    } else {
      switch (game_data->current_player) {
      case PLAYER_HUMAN:
        if (human_choice_position == 0) {
          /* Probably just starting the pegging and the human has not had
           * a chance to pick a card. Let them pick one.
           */
          return ADVANCE_RESULT_WAIT_FOR_USER;
        }
        /* Position count starts at 1, but hand placement starts at 0 */
        human_choice_position -= 1;
        played_card =
            game_data->player_hands[PLAYER_HUMAN][human_choice_position];
        if (!IS_CARD(played_card)) {
          /* Player must choose a valid card. Do not advance the game state. */
          return ADVANCE_RESULT_WAIT_FOR_USER;
        }
        if (played_card.value + game_data->pegging_count > 31) {
          /* Player can't choose a card that makes the pegging count exceed 31.
           * Do not advance the game state. */
          return ADVANCE_RESULT_WAIT_FOR_USER;
        }
        /* Alright, the player played a valid card. */
        game_data->player_hands[PLAYER_HUMAN][human_choice_position] =
            CARD_NONE;
        game_data->remaining_cards[PLAYER_HUMAN] -= 1;
        break;
      case PLAYER_CPU:
        /* By the time we get here, we know the CPU has at least one valid play.
         * Find it and play it.
         * TODO: Implement actual play logic here
         */
        for (int i = 0; i < 4; i++) {
          if (IS_CARD(game_data->player_hands[PLAYER_CPU][i]) &&
              (game_data->player_hands[PLAYER_CPU][i].rank +
                   game_data->pegging_count <=
               31)) {
            played_card = game_data->player_hands[PLAYER_CPU][i];
            game_data->player_hands[PLAYER_CPU][i] = CARD_NONE;
            game_data->remaining_cards[PLAYER_CPU] -= 1;
            break;
          }
        }
        break;
      default:
        break;
      }
    }
  }
  if (IS_CARD(played_card)) {
    *game_data->current_played_position = played_card;
    game_data->current_played_position++;
    /* Ensure that the end of the play sequence is marked. */
    *game_data->current_played_position = CARD_NONE;
    game_data->pegging_count += played_card.value;
    /* Score the pegging cards. */
    score_pegging(game_data->played_cards, game_data->score_list, 0);
    enum ScoreType *score = game_data->score_list;
    while (*score != SCORE_DONE) {
      game_data->scores[game_data->current_player] += SCORE_VALUES[*score];
      score++;
    }
    game_data->last_card_player = game_data->current_player;
  }
  game_data->current_player = get_next_player(game_data->current_player);

  /* We want to continue and have the simulator figure out the last card
   * situation, which is handled at the top.
   */
  return ADVANCE_RESULT_CONTINUE;
}

enum GameAdvanceResult
game_data_handle_state_announce_last_card(struct GameData *game_data,
                                          int human_choice_position) {
  if (game_data->remaining_cards[PLAYER_HUMAN] == 0 &&
      game_data->remaining_cards[PLAYER_CPU] == 0) {
    game_data_transition(game_data, STATE_COUNTING);
    return ADVANCE_RESULT_CONTINUE;
  }
  game_data_transition(game_data, STATE_PEGGING);
  return ADVANCE_RESULT_CONTINUE;
}

enum GameAdvanceResult
game_data_handle_state_counting(struct GameData *game_data,
                                int human_choice_position) {
  return ADVANCE_RESULT_WAIT_FOR_USER;
}

enum GameAdvanceResult (*state_handlers[STATE_END])(struct GameData *, int) = {
    &game_data_handle_state_choose_dealer,
    &game_data_handle_state_announce_dealer,
    &game_data_handle_state_choose_crib,
    &game_data_handle_state_announce_nibs,
    &game_data_handle_state_pegging,
    &game_data_handle_state_announce_last_card,
    &game_data_handle_state_announce_last_card,
    &game_data_handle_state_counting};

/* This advances the game state. GameData is opaque to the rest of the
 * program, so we can be sure that only this function is changing
 * game state
 */
enum GameAdvanceResult game_data_advance_game(struct GameData *game_data,
                                              int human_choice_position) {
  g_log(G_LOG_DOMAIN, G_LOG_LEVEL_DEBUG,
        "Advancing game state. Choice position is %d and current player is %s",
        human_choice_position, player_type_names[game_data->current_player]);
  enum GameAdvanceResult result =
      state_handlers[game_data->state](game_data, human_choice_position);
  g_log(G_LOG_DOMAIN, G_LOG_LEVEL_DEBUG,
        "Game state advanced. Result is %s and current player is %s",
        advance_result_names[result],
        player_type_names[game_data->current_player]);
  return result;
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
    scene->deck_cut_scene.human_card = game_data->player_hands[PLAYER_HUMAN][0];
    scene->deck_cut_scene.chosen_slot = game_data->cut_card_positions[0];
    break;
  case STATE_ANNOUNCE_DEALER:
    scene->type = ANNOUNCE_DEALER_SCENE;
    scene->announce_dealer_scene.chosen_cards[PLAYER_HUMAN] =
        game_data->player_hands[PLAYER_HUMAN][0];
    scene->announce_dealer_scene.chosen_cards[PLAYER_CPU] =
        game_data->player_hands[PLAYER_CPU][0];
    scene->announce_dealer_scene.chosen_slots[0] =
        game_data->cut_card_positions[0];
    scene->announce_dealer_scene.chosen_slots[1] =
        game_data->cut_card_positions[1];
    scene->announce_dealer_scene.first_dealer = game_data->dealer;
    break;
  case STATE_CHOOSE_CRIB:
    scene->type = CHOOSE_CRIB_SCENE;
    for (int i = 0; i < 6; i++) {
      scene->choose_crib_scene.human_cards[i] =
          game_data->player_hands[PLAYER_HUMAN][i];
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
      scene->announce_nibs_scene.human_cards[i] =
          game_data->player_hands[PLAYER_HUMAN][i];
      scene->announce_nibs_scene.up_card = game_data->up_card;
    }
    scene->announce_nibs_scene.scores[PLAYER_HUMAN] =
        game_data->scores[PLAYER_HUMAN];
    scene->announce_nibs_scene.scores[PLAYER_CPU] =
        game_data->scores[PLAYER_CPU];
    scene->announce_nibs_scene.dealer = game_data->dealer;
    break;
  case STATE_PEGGING:
  case STATE_ANNOUNCE_LAST_CARD:
    scene->type = PEGGING_SCENE;
    for (int i = 0; i < 4; i++) {
      scene->pegging_scene.human_cards[i] =
          game_data->player_hands[PLAYER_HUMAN][i];
      scene->pegging_scene.up_card = game_data->up_card;
    }
    for (int i = 0; i < PLAYER_END; i++) {
      scene->pegging_scene.called_go[i] = game_data->called_go[i];
    }
    scene->pegging_scene.scores[PLAYER_HUMAN] = game_data->scores[PLAYER_HUMAN];
    scene->pegging_scene.scores[PLAYER_CPU] = game_data->scores[PLAYER_CPU];
    scene->pegging_scene.dealer = game_data->dealer;

    struct Card *played, *scene_cards;
    played = game_data->played_cards;
    scene_cards = scene->pegging_scene.played_cards;
    while (IS_CARD((*played))) {
      *scene_cards = *played;
      scene_cards++;
      played++;
    }
    *scene_cards = CARD_NONE;
    scene->pegging_scene.pegging_count = game_data->pegging_count;
    scene->pegging_scene.current_player = game_data->current_player;
    scene->pegging_scene.remaining_cpu_cards =
        game_data->remaining_cards[PLAYER_CPU];
    scene->pegging_scene.last_card =
        game_data->state == STATE_ANNOUNCE_LAST_CARD;
    scene->pegging_scene.last_card_player = game_data->last_card_player;
    break;
  default:
    scene->type = BLANK_SCENE;
  }
}

struct GameData *game_data_create() {
  /* Not the best place to initialize random number seed. */
  srand(time(0));

  struct GameData *game_data =
      (struct GameData *)calloc(sizeof(struct GameData), 1);
  game_data_transition(game_data, STATE_CHOOSE_DEALER);
  return game_data;
}

void game_data_destroy(struct GameData *game_data) {
  if (game_data) {
    free(game_data);
  }
}
