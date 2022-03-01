#pragma once

enum GameState {
    NOT_STARTED,
    CHOOSE_DEALER,
    PEGGING,
    COUNTING,
    DONE
};

struct GameData {
    enum GameState state;
};
