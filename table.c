#include <cairo.h>
#include <gtk/gtk.h>
#include "table.h"
#include "simulation.h"

#define MAX_HITBOXES 13
#define CARD_MAX_CUT_POSITIONS 13

struct CardOptions {
    int width;
    int height;
    int fan_spacing;
    int top_offset;
    int middle_offset;
    int bottom_offset;
};

struct HitBox {
    int x;
    int y;
    int width;
    int height;
};

struct _GCribbageTable {
    GtkWidget parent;
    struct GameData *game_data;
    cairo_t *buffer_context;
    GdkPixbuf *card_images;
    GdkPixbuf *card_back;
    struct HitBox hit_boxes[MAX_HITBOXES];
    int num_hitboxes;
    struct CardOptions card_options;
};

G_DEFINE_TYPE(GCribbageTable, gcribbage_table, GTK_TYPE_DRAWING_AREA);

GdkRGBA BACKGROUND_COLOR = { 0.f, 0.6f, 0.f, 1.f };

void clear_buffer(cairo_t *cr) {
    gdk_cairo_set_source_rgba(cr, &BACKGROUND_COLOR);
    cairo_paint(cr);
}

void gcribbage_table_render_card(GCribbageTable *table, char card, int x, int y) {
    int rank = (card & 0xf) - 1;  // Rank goes from 1 - 13 but we want to offset from 0
    int suit = (card >> 4);
    int src_x = table->card_options.width * rank;
    int src_y = table->card_options.height * suit;
    gdk_cairo_set_source_pixbuf(
        table->buffer_context,
        table->card_images,
        x - src_x,
        y - src_y
    );
    cairo_rectangle(
        table->buffer_context,
        x,
        y,
        table->card_options.width,
        table->card_options.height
    );
    cairo_fill(table->buffer_context);
}

void gcribbage_table_render_card_back(GCribbageTable *table, int x, int y) {
    gdk_cairo_set_source_pixbuf(
        table->buffer_context,
        table->card_back,
        x,
        y
    );
    cairo_rectangle(
        table->buffer_context,
        x,
        y,
        table->card_options.width,
        table->card_options.height
    );
    cairo_fill(table->buffer_context);
}

/* Figures out how to center text and draws it on the table's buffer.
 * The y value is where you want the *BOTTOM* of the text to be.
 * TODO: Make this take in the lane and position for the text
 */
void gcribbage_table_render_centered_text(GCribbageTable *table, char *text, int y) {
    cairo_surface_t *surface = cairo_get_target(table->buffer_context);
    int win_width = cairo_image_surface_get_width(surface);
    cairo_text_extents_t extents;
    cairo_text_extents(table->buffer_context, text, &extents);
    int x = win_width / 2 - (int)extents.width / 2;
    y += (int)extents.height;
    cairo_set_source_rgb(table->buffer_context, 0.8, 0.8, 0.8);
    cairo_move_to(table->buffer_context, x, y);
    cairo_show_text(table->buffer_context, text);
}

void gcribbage_table_render_choose_dealer(GCribbageTable *table, char player_chosen_card, char cpu_chosen_card) {
    struct HitBox *hitbox;
    int width = table->card_options.fan_spacing * CARD_MAX_CUT_POSITIONS - 1 + table->card_options.width;
    cairo_surface_t *surface = cairo_get_target(table->buffer_context);
    int win_width = cairo_image_surface_get_width(surface);

    width = win_width / 2 - width / 2;

    // Reset hitbox count to 0
    table->num_hitboxes = 0;

    for (int i = 0; i < CARD_MAX_CUT_POSITIONS; i++) {
        gcribbage_table_render_card_back(
            table,
            width + i * table->card_options.fan_spacing,
            table->card_options.top_offset
        );
        hitbox = &table->hit_boxes[table->num_hitboxes];
        if (i < CARD_MAX_CUT_POSITIONS - 1) {
            hitbox->x = width + i * table->card_options.fan_spacing;
            hitbox->y = table->card_options.top_offset;
            hitbox->width = table->card_options.fan_spacing;
            hitbox->height = table->card_options.height;
        } else {
            hitbox->x = width + i * table->card_options.fan_spacing;
            hitbox->y = table->card_options.top_offset;
            hitbox->width = table->card_options.width;
            hitbox->height = table->card_options.height;
        }
        table->num_hitboxes++;
    }

    if (player_chosen_card != CARD_NONE) {
        gcribbage_table_render_card(
            table,
            player_chosen_card,
            width,
            table->card_options.middle_offset
        );

        gcribbage_table_render_card(
            table,
            cpu_chosen_card,
            width + table->card_options.fan_spacing * 12,
            table->card_options.middle_offset
        );

        if ((player_chosen_card & 0xf) < (cpu_chosen_card & 0xf)) {
            gcribbage_table_render_centered_text(
                table,
                "You deal first.",
                table->card_options.middle_offset
            );
        } else {
            gcribbage_table_render_centered_text(
                table,
                "CPU deals first.",
                table->card_options.middle_offset
            );
        }
    } else {
        gcribbage_table_render_centered_text(
            table,
            "Choose a card. Lowest card deals first.",
            table->card_options.middle_offset
        );
    }
}

void render_buffer(GCribbageTable *table) {
    struct RenderScene scene;
    game_data_get_render_scene(table->game_data, &scene);
    clear_buffer(table->buffer_context);
    switch (scene.type) {
        case DECK_CUT_SCENE:
            gcribbage_table_render_choose_dealer(
                table,
                scene.deck_cut_scene.player_card,
                scene.deck_cut_scene.cpu_card
            );
            break;
        default:
            break;
    }
}

void gcribbage_table_handle_resize(GCribbageTable *table, int width, int height, gpointer userdata) {
    if (table->buffer_context) {
        cairo_destroy(table->buffer_context);
    }
    cairo_surface_t *surface = cairo_image_surface_create(
        CAIRO_FORMAT_ARGB32, 
        width,
        height
    );
    table->buffer_context = cairo_create(surface);
    cairo_select_font_face(table->buffer_context, "sans-serif", CAIRO_FONT_SLANT_NORMAL, CAIRO_FONT_WEIGHT_NORMAL);
    cairo_set_font_size(table->buffer_context, 18);
    table->card_options.top_offset = 25;
    table->card_options.middle_offset = height / 2 - table->card_options.height / 2;
    table->card_options.bottom_offset = height - 25 - table->card_options.height;
    render_buffer(table);
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

static void gcribbage_table_advance_game(GCribbageTable *table, int player_position_choice) {
    game_data_advance_game(table->game_data, player_position_choice);
    render_buffer(table);
    gtk_widget_queue_draw(GTK_WIDGET(table));
}

static void pressed(GtkGestureClick *gesture, int n_press, double x, double y, GtkWidget *area) {
    struct HitBox *hitbox;
    GCribbageTable *table = GCRIBBAGE_TABLE(area);
    for(int i = 0; i < table->num_hitboxes; i++) {
        hitbox = &table->hit_boxes[i];
        if (x > hitbox->x && x < hitbox->x + hitbox->width && y > hitbox->y && y < hitbox->y + hitbox->height) {
            gcribbage_table_advance_game(table, i + 1);
        }
    }
}

void gcribbage_table_start_new_game(GCribbageTable *table) {
    table->game_data = game_data_create();
}

static void gcribbage_table_init(GCribbageTable *table) {
    GtkGesture *press;
    press = gtk_gesture_click_new();
    gtk_gesture_single_set_button(GTK_GESTURE_SINGLE(press), GDK_BUTTON_PRIMARY);
    gtk_widget_add_controller(GTK_WIDGET(table), GTK_EVENT_CONTROLLER(press));
    g_signal_connect(press, "pressed", G_CALLBACK(pressed), table);
    g_signal_connect(table, "resize", G_CALLBACK(gcribbage_table_handle_resize), NULL);
    GError *error = NULL;
    table->card_images = gdk_pixbuf_new_from_resource(
        "/com/ronsbrain/gcribbage/assets/cards.png",
        &error
    );
    table->card_back = gdk_pixbuf_new_from_resource(
        "/com/ronsbrain/gcribbage/assets/back.png",
        &error
    );
    table->card_options.width = gdk_pixbuf_get_width(table->card_images) / 13;
    table->card_options.height = gdk_pixbuf_get_height(table->card_images) / 4;
    table->card_options.fan_spacing = 30;
    gtk_drawing_area_set_draw_func(
        GTK_DRAWING_AREA(table),
        draw,
        (gpointer)table,
        NULL
    );
    table->game_data = game_data_create();
}

static void gcribbage_table_class_init (GCribbageTableClass *klass) {
}
