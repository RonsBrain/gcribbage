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
