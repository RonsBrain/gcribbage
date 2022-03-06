#include <gtk/gtk.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include "simulation.h"

enum GameState {
    STATE_CHOOSE_DEALER,
    STATE_PEGGING,
    STATE_COUNTING,
    STATE_WINNER
};

struct GameData {
    enum GameState state;
    char player_chosen_cards[6];
    char cpu_chosen_cards[6];
};

char possible_cards[52] = {};

/* Uses rejection sampling to return a random value between min and max inclusive */
int get_random_number(int min, int max) {
    int limit = RAND_MAX - (RAND_MAX % (max - min));
    int r;
    while((r = rand()) >= limit);
    return r % max + min;
}

void get_random_cards(int num_cards, char *cards) {
    char *current = cards;
    for (int i = 0; i < num_cards; i++) {
        *current = possible_cards[get_random_number(0, 51)];
        current++;
    }
}

struct GameData *game_data_create() {
    /* Not the best place to initialize random number seed. */
    srand(time(0));
    
    /*
     * A card is represented by six bytes. The lower four represent the rank,
     * and the upper two represents the suit.
     *
     * To check the suit of a card, you will need to do compare only
     * the upper two bits, but for cribbage, knowing the exact suit is
     * not important for scoring, only that cards are the same suit.
     */

    /* We can probably find a better place for this too. */
    if (possible_cards[0] == CARD_NONE) {
        int i = 0;
        for (int suit = 0; suit <= 4; suit++) {
            for (int rank = 1; rank < 14; rank++) {
                possible_cards[i] = (suit << 4) | rank;
                i++;
            }
        }
    }

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
    switch (game_data->state) {
        case (STATE_CHOOSE_DEALER):
            if (player_choice_position != POSITION_NONE) {
                char chosen[2];
                get_random_cards(2, chosen);
                game_data->player_chosen_cards[0] = chosen[0];
                game_data->cpu_chosen_cards[0] = chosen[1];
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
    scene->type = DECK_CUT_SCENE;
    scene->deck_cut_scene.player_card = game_data->player_chosen_cards[0];
    scene->deck_cut_scene.cpu_card = game_data->cpu_chosen_cards[0];
    scene->deck_cut_scene.chosen_slots[0] = 0;
    scene->deck_cut_scene.chosen_slots[1] = 0;
}
