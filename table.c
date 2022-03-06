#include <cairo.h>
#include <gtk/gtk.h>
#include "table.h"
#include "simulation.h"

#define MAX_HITBOXES 13
#define CARD_DIMENSIONS_WIDTH 150
#define CARD_DIMENSIONS_HEIGHT 210
#define CARD_FAN_SPACING 30
#define CARD_CHOOSE_DEALER_OFFSET 50
#define CARD_MAX_POSITIONS 13

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
};

G_DEFINE_TYPE(GCribbageTable, gcribbage_table, GTK_TYPE_DRAWING_AREA);

GdkRGBA BACKGROUND_COLOR = { 0.f, 0.6f, 0.f, 1.f };

void clear_buffer(cairo_t *cr) {
    gdk_cairo_set_source_rgba(cr, &BACKGROUND_COLOR);
    cairo_paint(cr);
}

void render_card(cairo_t *cr, GdkPixbuf *card_images, int rank, int suit, int x, int y) {
    int src_x = CARD_DIMENSIONS_WIDTH * rank;
    int src_y = CARD_DIMENSIONS_HEIGHT * suit;
    gdk_cairo_set_source_pixbuf(
        cr,
        card_images,
        x - src_x,
        y - src_y
    );
    cairo_rectangle(cr, x, y, CARD_DIMENSIONS_WIDTH, CARD_DIMENSIONS_HEIGHT);
    cairo_fill(cr);
}

void render_card_back(cairo_t *cr, GdkPixbuf *card_back, int x, int y) {
    gdk_cairo_set_source_pixbuf(
        cr,
        card_back,
        x,
        y
    );
    cairo_rectangle(cr, x, y, CARD_DIMENSIONS_WIDTH, CARD_DIMENSIONS_HEIGHT);
    cairo_fill(cr);
}

void render_choose_dealer(GCribbageTable *table, char player_chosen_card, char cpu_chosen_card) {
    struct HitBox *hitbox;
    int width = CARD_FAN_SPACING * 12 + CARD_DIMENSIONS_WIDTH;
    cairo_surface_t *surface = cairo_get_target(table->buffer_context);
    int win_width = cairo_image_surface_get_width(surface);

    width = win_width / 2 - width / 2;

    // Reset hitbox count to 0
    table->num_hitboxes = 0;

    for (int i = 0; i < CARD_MAX_POSITIONS; i++) {
        render_card_back(
            table->buffer_context,
            table->card_back,
            width + i * CARD_FAN_SPACING,
            CARD_CHOOSE_DEALER_OFFSET
        );
        hitbox = &table->hit_boxes[table->num_hitboxes];
        if (i < CARD_MAX_POSITIONS - 1) {
            hitbox->x = width + i * CARD_FAN_SPACING;
            hitbox->y = CARD_CHOOSE_DEALER_OFFSET;
            hitbox->width = CARD_FAN_SPACING;
            hitbox->height = CARD_DIMENSIONS_HEIGHT;
        } else {
            hitbox->x = width + i * CARD_FAN_SPACING;
            hitbox->y = CARD_CHOOSE_DEALER_OFFSET;
            hitbox->width = CARD_DIMENSIONS_WIDTH;
            hitbox->height = CARD_DIMENSIONS_HEIGHT;
        }
        table->num_hitboxes++;
    }

    if (player_chosen_card != CARD_NONE) {
        int rank, suit;
        rank = player_chosen_card & 0xf;
        suit = player_chosen_card & (1 << 4) >> 4;
        render_card(
            table->buffer_context,
            table->card_images,
            rank,
            suit,
            width,
            CARD_CHOOSE_DEALER_OFFSET * 2 + CARD_DIMENSIONS_HEIGHT + CARD_FAN_SPACING
        );

        rank = cpu_chosen_card & 0xf;
        suit = cpu_chosen_card & (1 << 4) >> 4;
        render_card(
            table->buffer_context,
            table->card_images,
            rank,
            suit,
            width + CARD_DIMENSIONS_WIDTH + CARD_FAN_SPACING,
            CARD_CHOOSE_DEALER_OFFSET * 2 + CARD_DIMENSIONS_HEIGHT + CARD_FAN_SPACING
        );

        cairo_set_source_rgb(table->buffer_context, 0.8, 0.8, 0.8);
        cairo_select_font_face(table->buffer_context, "sans-serif", CAIRO_FONT_SLANT_NORMAL, CAIRO_FONT_WEIGHT_NORMAL);
        cairo_set_font_size(table->buffer_context, 18);
        cairo_move_to(table->buffer_context, width, CARD_CHOOSE_DEALER_OFFSET * 2 + CARD_DIMENSIONS_HEIGHT);
        if ((player_chosen_card & 0xf) < (cpu_chosen_card & 0xf)) {
            cairo_show_text(table->buffer_context, "You deal first.");
        } else {
            cairo_show_text(table->buffer_context, "CPU deals first.");
        }
    } else {
        cairo_set_source_rgb(table->buffer_context, 0.8, 0.8, 0.8);
        cairo_select_font_face(table->buffer_context, "sans-serif", CAIRO_FONT_SLANT_NORMAL, CAIRO_FONT_WEIGHT_NORMAL);
        cairo_set_font_size(table->buffer_context, 18);
        cairo_move_to(table->buffer_context, width, CARD_CHOOSE_DEALER_OFFSET * 2 + CARD_DIMENSIONS_HEIGHT);
        cairo_show_text(table->buffer_context, "Choose a card. Lowest card deals first.");
    }
}

void render_buffer(GCribbageTable *table) {
    struct RenderScene scene;
    game_data_get_render_scene(table->game_data, &scene);
    clear_buffer(table->buffer_context);
    switch (scene.type) {
        case DECK_CUT_SCENE:
            render_choose_dealer(
                table,
                scene.deck_cut_scene.player_card,
                scene.deck_cut_scene.cpu_card
            );
            break;
        default:
            break;
    }
}

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

static void pressed(GtkGestureClick *gesture, int n_press, double x, double y, GtkWidget *area) {
    struct HitBox *hitbox;
    GCribbageTable *table = GCRIBBAGE_TABLE(area);
    for(int i = 0; i < table->num_hitboxes; i++) {
        hitbox = &table->hit_boxes[i];
        if (x > hitbox->x && x < hitbox->x + hitbox->width && y > hitbox->y && y < hitbox->y + hitbox->height) {
            g_log(G_LOG_DOMAIN, G_LOG_LEVEL_DEBUG, "Position %d clicked", i + 1);
            gcribbage_table_update_game_data(table, table->game_data, i + 1);
        }
    }
}

void gcribbage_table_update_game_data(GCribbageTable *table, struct GameData *game_data, int player_choice_position) {
    g_log(G_LOG_DOMAIN, G_LOG_LEVEL_DEBUG, "Received update");
    table->game_data = game_data;
    game_data_advance_game(table->game_data, player_choice_position);
    render_buffer(table);
    gtk_widget_queue_draw(GTK_WIDGET(table));
}

static void gcribbage_table_init(GCribbageTable *table) {
    GtkGesture *press;
    press = gtk_gesture_click_new();
    gtk_gesture_single_set_button(GTK_GESTURE_SINGLE(press), GDK_BUTTON_PRIMARY);
    gtk_widget_add_controller(GTK_WIDGET(table), GTK_EVENT_CONTROLLER(press));
    g_signal_connect(press, "pressed", G_CALLBACK(pressed), table);
    g_signal_connect(table, "resize", G_CALLBACK(allocate_buffer), NULL);
    GError *error = NULL;
    table->card_images = gdk_pixbuf_new_from_resource(
        "/com/ronsbrain/gcribbage/assets/cards.png",
        &error
    );
    table->card_back = gdk_pixbuf_new_from_resource(
        "/com/ronsbrain/gcribbage/assets/back.png",
        &error
    );
    gtk_drawing_area_set_draw_func(
        GTK_DRAWING_AREA(table),
        draw,
        (gpointer)table,
        NULL
    );
}

static void gcribbage_table_class_init (GCribbageTableClass *klass) {
}
