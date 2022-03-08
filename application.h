#pragma once

#include <gtk/gtk.h>

#define GCRIBBAGE_APPLICATION_TYPE (gcribbage_application_get_type())
G_DECLARE_FINAL_TYPE(GCribbageApplication, gcribbage_application, GCRIBBAGE,
                     APPLICATION, GtkApplication)

GCribbageApplication *gcribbage_application_new(void);
struct GameData *gcribbage_application_new_game(GCribbageApplication *app);
