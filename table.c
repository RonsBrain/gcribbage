#include <cairo.h>
#include <gtk/gtk.h>
#include "table.h"
#include "simulation.h"

struct _GCribbageTable {
    GtkWidget parent;
    struct GameData *game_data;
};

G_DEFINE_TYPE(GCribbageTable, gcribbage_table, GTK_TYPE_DRAWING_AREA);

static void draw(GtkDrawingArea *table, cairo_t *cr, int width, int height, gpointer data) {
    g_log(G_LOG_DOMAIN, G_LOG_LEVEL_WARNING, "Drawing!");
    GdkRGBA color = { 0.f, 0.6f, 0.f, 1.f };
    gdk_cairo_set_source_rgba(cr, &color);
    cairo_paint(cr);
}

static void gcribbage_table_init(GCribbageTable *table) {
    gtk_drawing_area_set_draw_func(
        GTK_DRAWING_AREA(table),
        draw,
        (gpointer)table->game_data,
        NULL
    );
}

static void gcribbage_table_class_init (GCribbageTableClass *klass) {
}
