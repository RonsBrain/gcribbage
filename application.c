#include <gtk/gtk.h>
#include "application.h"
#include "window.h"
#include "simulation.h"

struct _GCribbageApplication {
    GtkApplication parent;
    struct GameData *game_data;
};

G_DEFINE_TYPE(GCribbageApplication, gcribbage_application, GTK_TYPE_APPLICATION);

struct GameData *gcribbage_application_new_game(GCribbageApplication *app) {
    g_free((gpointer)app->game_data);
    app->game_data = (struct GameData *)g_malloc0(sizeof(struct GameData));
    return app->game_data;
}

static void quit_activated(
    GSimpleAction *action,
    GVariant *parameter,
    gpointer app
) {
    g_application_quit(G_APPLICATION(app));
}

static void gcribbage_application_init(GCribbageApplication *app) {
}

static void gcribbage_application_activate(GApplication *app) {
    GCribbageApplicationWindow *win;

    win = gcribbage_application_window_new(GCRIBBAGE_APPLICATION(app));
    gcribbage_application_new_game(GCRIBBAGE_APPLICATION(app));
    gtk_window_present(GTK_WINDOW(win));
}

static GActionEntry app_entries[] = {
    {"quit", quit_activated, NULL, NULL, NULL}
};

static void gcribbage_application_startup(GApplication *app) {
    const char *quit_accels[2] = { "<CTRL>Q", NULL };

    G_APPLICATION_CLASS(gcribbage_application_parent_class)->startup(app);

    g_action_map_add_action_entries(
        G_ACTION_MAP(app),
        app_entries,
        G_N_ELEMENTS(app_entries),
        app
    );

    gtk_application_set_accels_for_action(
        GTK_APPLICATION(app),
        "app.quit",
        quit_accels
    );
}

static void gcribbage_application_class_init(GCribbageApplicationClass *klass) {
    G_APPLICATION_CLASS(klass)->activate = gcribbage_application_activate;
    G_APPLICATION_CLASS(klass)->startup = gcribbage_application_startup;
}

GCribbageApplication *gcribbage_application_new(void) {
    return (GCribbageApplication *)g_object_new(
            GCRIBBAGE_APPLICATION_TYPE,
            "application-id", "com.ronsbrain.gcribbage",
            "flags", G_APPLICATION_FLAGS_NONE,
            NULL
    );
}
