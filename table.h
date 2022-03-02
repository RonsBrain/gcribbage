#pragma once

#include <gtk/gtk.h>
#include "simulation.h"

#define GCRIBBAGE_TABLE_TYPE (gcribbage_table_get_type())
G_DECLARE_FINAL_TYPE(GCribbageTable, gcribbage_table, GCRIBBAGE, TABLE, GtkDrawingArea)

GCribbageTable *gcribbage_table_new(void);
void gcribbage_table_update_game_data(GCribbageTable *table, struct GameData *game_data);
