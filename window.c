#include <gtk/gtk.h>

#include "application.h"
#include "simulation.h"
#include "table.h"
#include "window.h"

struct _GCribbageApplicationWindow {
    GtkApplicationWindow parent;
    GtkWidget *menu_button;
    GtkWidget *table;
};

G_DEFINE_TYPE(GCribbageApplicationWindow, gcribbage_application_window, GTK_TYPE_APPLICATION_WINDOW);

void new_game_clicked(GtkButton *button, GCribbageApplicationWindow *win) {
    struct GameData *game_data;
    game_data = gcribbage_application_new_game(GCRIBBAGE_APPLICATION(gtk_window_get_application(GTK_WINDOW(win))));
    gcribbage_table_update_game_data(GCRIBBAGE_TABLE(win->table), game_data);
}

static void gcribbage_application_window_init(GCribbageApplicationWindow *win) {
    GtkBuilder *builder;
    GMenuModel *menu;

    gtk_widget_init_template(GTK_WIDGET(win));

    builder = gtk_builder_new_from_resource("/com/ronsbrain/gcribbage/menu.ui");
    menu = G_MENU_MODEL(gtk_builder_get_object(builder, "menu"));
    gtk_menu_button_set_menu_model(GTK_MENU_BUTTON(win->menu_button), menu);

    g_object_unref(builder);
}

static void gcribbage_application_window_class_init (GCribbageApplicationWindowClass *klass) {
    g_type_ensure(GCRIBBAGE_TABLE_TYPE);
    gtk_widget_class_set_template_from_resource(
        GTK_WIDGET_CLASS(klass),
        "/com/ronsbrain/gcribbage/window.ui"
    );
    gtk_widget_class_bind_template_callback(
        GTK_WIDGET_CLASS(klass),
        new_game_clicked
    );
    gtk_widget_class_bind_template_child(
        GTK_WIDGET_CLASS(klass),
        GCribbageApplicationWindow,
        menu_button
    );
    gtk_widget_class_bind_template_child(
        GTK_WIDGET_CLASS(klass),
        GCribbageApplicationWindow,
        table
    );
}

GCribbageApplicationWindow *gcribbage_application_window_new(GCribbageApplication *app) {
    return (GCribbageApplicationWindow *)g_object_new(
        GCRIBBAGE_APPLICATION_WINDOW_TYPE,
        "application", app,
        NULL
    );
}
