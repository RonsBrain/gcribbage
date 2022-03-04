#pragma once

enum GameState {
    CHOOSE_DEALER,
    PEGGING,
    COUNTING,
    DONE
};

struct GameData {
    enum GameState state;
};
