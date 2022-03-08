#include <stdlib.h>
#include <string.h>
#include <time.h>
#include "simulation.h"

enum GameState {
    STATE_CHOOSE_DEALER,
    STATE_CHOOSE_CRIB,
    STATE_PEGGING,
    STATE_COUNTING,
    STATE_WINNER
};

struct GameData {
    enum GameState state;
    char player_hand[6];
    char cpu_hand[6];
    char crib_hand[4];
    char cut_cards[2];
    char up_card;
    int player_crib_choices[2];
};

/*
 * A card is represented by six bytes. The lower four represent the rank,
 * and the upper two represents the suit.
 *
 * To check the suit of a card, you will need to do compare only
 * the upper two bits, but for cribbage, knowing the exact suit is
 * not important for scoring, only that cards are the same suit.
 */
char possible_cards[52] = {
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
    0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d,
    0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d,
    0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d
};

/* Uses rejection sampling to return a random value between min and max inclusive */
int get_random_number(int min, int max) {
    int limit = RAND_MAX - (RAND_MAX % (max - min));
    int r;
    while((r = rand()) >= limit);
    return r % max + min;
}

void get_random_cards(int num_cards, char *cards) {
    char *current = cards;
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
    
    /* 0 is a valid value for all initial values of GameData.
     * If this ever becomes not the case, then the allocated memory
     * should be initialized before returning it.
     */
    return (struct GameData *)calloc(sizeof(struct GameData), 1);
}

void game_data_destroy(struct GameData *game_data) {
    if(game_data) {
        free(game_data);
    }
}

/* This advances the game state. GameData is opaque to the rest of the
 * program, so we can be sure that only this function is changing
 * game state
 */
void game_data_advance_game(struct GameData *game_data, int player_choice_position) {
    int ready_to_proceed;
    switch (game_data->state) {
        case (STATE_CHOOSE_DEALER):
            if (player_choice_position != POSITION_NONE) {
                char chosen[2];
                get_random_cards(2, chosen);
                game_data->player_hand[0] = chosen[0];
                game_data->cpu_hand[0] = chosen[1];
                game_data->cut_cards[0] = player_choice_position;
                game_data->cut_cards[1] = get_random_number(1, 13);
            } else {
                if (game_data->cut_cards[0] != POSITION_NONE) {
                    /* Cards already cut, move on to crib building. */
                    game_data->state = STATE_CHOOSE_CRIB;
                    char cards[13];
                    get_random_cards(13, cards);
                    for (int i = 0; i < 6; i++) {
                        game_data->player_hand[i] = cards[i];
                        game_data->cpu_hand[i] = cards[i + 6];
                    }
                    game_data->up_card = cards[12];
                }
            }
            break;
        case (STATE_CHOOSE_CRIB):
            ready_to_proceed = (
                game_data->player_crib_choices[0] != POSITION_NONE &&
                game_data->player_crib_choices[1] != POSITION_NONE
            );

            if (player_choice_position != POSITION_NONE) {
                if  (ready_to_proceed) {
                    /* Two cards already chosen, choosing another is illegal.
                     * Do not advance the game.
                     */
                    return;
                }
                int removed = 0;
                for (int i = 0; i < 2; i++) {
                    if (game_data->player_crib_choices[i] == player_choice_position) {
                        game_data->player_crib_choices[i] = POSITION_NONE;
                        removed = 1;
                    }
                }
                if (!removed) {
                   if (game_data->player_crib_choices[0] == POSITION_NONE) {
                       game_data->player_crib_choices[0] = player_choice_position;
                   } else {
                       game_data->player_crib_choices[1] = player_choice_position;
                   }
                }
            } else {
                if (!ready_to_proceed) {
                    for (int i = 0; i < 2; i++) {
                        game_data->player_crib_choices[i] = 0;
                    }
                } else {
                    game_data->state = STATE_PEGGING;
                    for (int i = 0; i < 2; i++) {
                        game_data->crib_hand[i] = game_data->player_hand[game_data->player_crib_choices[i]];
                        game_data->player_hand[game_data->player_crib_choices[i]] = 0;
                        /* TODO: Implement actual CPU AI instead of random crib choice */
                        game_data->crib_hand[i + 2] = game_data->cpu_hand[i];
                    }
                }
            }
            break;
        default:
            break;
    }
}

/* Since GameData is opaque, we need to give information about what
 * scene should be rendered.
 *
 * Each scene type contains all the information necessary to render a scene.
 * The rendering widget needs to figure out how to do the rendering.
 */
void game_data_get_render_scene(struct GameData *game_data, struct RenderScene *scene) {
    if (!game_data) {
        scene->type = BLANK_SCENE;
        return;
    }
    switch (game_data->state) {
        case STATE_CHOOSE_DEALER:
            scene->type = DECK_CUT_SCENE;
            scene->deck_cut_scene.player_card = game_data->player_hand[0];
            scene->deck_cut_scene.cpu_card = game_data->cpu_hand[0];
            scene->deck_cut_scene.chosen_slots[0] = game_data->cut_cards[0];
            scene->deck_cut_scene.chosen_slots[1] = game_data->cut_cards[1];
            break;
        case STATE_CHOOSE_CRIB:
            scene->type = CHOOSE_CRIB_SCENE;
            for (int i = 0; i < 6; i++) {
                scene->choose_crib_scene.player_cards[i] = game_data->player_hand[i];
            }
            scene->choose_crib_scene.player_crib_choices[0] = game_data->player_crib_choices[0];
            scene->choose_crib_scene.player_crib_choices[1] = game_data->player_crib_choices[1];
            scene->choose_crib_scene.ready_to_proceed = (game_data->player_crib_choices[0] != POSITION_NONE && game_data->player_crib_choices[1] != POSITION_NONE);
            break;
        default:
            scene->type = BLANK_SCENE;
    }
}
