#include <cairo.h>
#include <gtk/gtk.h>
#include "table.h"
#include "simulation.h"

struct _GCribbageTable {
    GtkWidget parent;
    struct GameData *game_data;
    cairo_t *buffer_context;
};

G_DEFINE_TYPE(GCribbageTable, gcribbage_table, GTK_TYPE_DRAWING_AREA);

GdkRGBA BACKGROUND_COLOR = { 0.f, 0.6f, 0.f, 1.f };

void allocate_buffer(GCribbageTable *table, int width, int height, gpointer userdata) {
    if (table->buffer_context) {
        cairo_destroy(table->buffer_context);
    }
    cairo_surface_t *surface = cairo_image_surface_create(
        CAIRO_FORMAT_ARGB32, 
        width,
        height
    );
    table->buffer_context = cairo_create(surface);
    gdk_cairo_set_source_rgba(table->buffer_context, &BACKGROUND_COLOR);
    cairo_paint(table->buffer_context);
}

static void draw(GtkDrawingArea *table, cairo_t *cr, int width, int height, gpointer data) {
    cairo_set_source_surface(
        cr,
        cairo_get_target(GCRIBBAGE_TABLE(table)->buffer_context),
        0,
        0
    );
    cairo_paint(cr);
}

void gcribbage_table_update_game_data(GCribbageTable *table, struct GameData *game_data) {
    table->game_data = game_data;
    gtk_widget_queue_draw(GTK_WIDGET(table));
}

static void gcribbage_table_init(GCribbageTable *table) {
    g_signal_connect(table, "resize", G_CALLBACK(allocate_buffer), NULL);
    gtk_drawing_area_set_draw_func(
        GTK_DRAWING_AREA(table),
        draw,
        (gpointer)table,
        NULL
    );
}

static void gcribbage_table_class_init (GCribbageTableClass *klass) {
}
