#pragma once

#include <gtk/gtk.h>
#include "application.h"
#include "simulation.h"

#define GCRIBBAGE_APPLICATION_WINDOW_TYPE (gcribbage_application_window_get_type())
G_DECLARE_FINAL_TYPE (GCribbageApplicationWindow, gcribbage_application_window, GCRIBBAGE, APPLICATION_WINDOW, GtkApplicationWindow)

GCribbageApplicationWindow *gcribbage_application_window_new(GCribbageApplication *app);
void gcribbage_application_window_update_game_data(GCribbageApplicationWindow *win, struct GameData *game_data);
