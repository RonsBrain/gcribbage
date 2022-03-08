#pragma once

#include "simulation.h"
#include <gtk/gtk.h>

#define GCRIBBAGE_TABLE_TYPE (gcribbage_table_get_type())
G_DECLARE_FINAL_TYPE(GCribbageTable, gcribbage_table, GCRIBBAGE, TABLE,
                     GtkDrawingArea)

GCribbageTable *gcribbage_table_new(void);
void gcribbage_table_start_new_game(GCribbageTable *table);
