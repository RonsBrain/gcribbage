#pragma once

#define CARD_NONE 0
#define POSITION_NONE 0
#define MAX_INSTRUCTIONS 32

struct GameData;

enum RenderType {
    BLANK_SCENE,
    DECK_CUT_SCENE
};

struct BlankScene {
};

struct RenderDeckCutScene {
    char player_card;
    char cpu_card;
    int chosen_slots[2];
};

struct RenderScene {
    enum RenderType type;
    union {
        struct BlankScene blank_scene;
        struct RenderDeckCutScene deck_cut_scene;
    };
};

struct GameData *game_data_create();
void game_data_destroy(struct GameData *game_data);
void game_data_advance_game(struct GameData *game_data, int player_choice_position);
void game_data_get_render_scene(struct GameData *game_data, struct RenderScene *scene);
