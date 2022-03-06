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
    int advance_data;
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

#define PI 3.14159265358979323846
void gcribbage_table_render_rounded_rectangle(
        GCribbageTable *table,
        double x,
        double y,
        double width,
        double height,
        double radius,
        double r,
        double g,
        double b) {
    cairo_new_sub_path(table->buffer_context);

    cairo_arc(table->buffer_context, x + radius, y + radius, radius, PI, 3 * PI / 2);
    cairo_arc(table->buffer_context, x + width - radius, y + radius, radius, 3 * PI / 2, 2 * PI);
    cairo_arc(table->buffer_context, x + width - radius, y + height - radius, radius, 0, PI / 2);
    cairo_arc(table->buffer_context, x + radius, y + height - radius, radius, PI / 2, PI);
    cairo_close_path(table->buffer_context);

    cairo_set_source_rgb(table->buffer_context, r, g, b);
    cairo_fill(table->buffer_context);
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

void gcribbage_table_add_hit_box(GCribbageTable *table, int x, int y, int width, int height, int data) {
    struct HitBox *hitbox;
    hitbox = &table->hit_boxes[table->num_hitboxes];
    hitbox->x = x;
    hitbox->y = y;
    hitbox->width = width;
    hitbox->height = height;
    hitbox->advance_data = data;
    table->num_hitboxes++;
}

#define DIALOG_PADDING 10
void gcribbage_table_render_dialog(GCribbageTable *table, char *text, int y, int show_ok) {
    cairo_surface_t *surface = cairo_get_target(table->buffer_context);
    int win_width = cairo_image_surface_get_width(surface);
    cairo_text_extents_t dialog_extents, ok_extents;
    cairo_text_extents(table->buffer_context, text, &dialog_extents);
    int x = win_width / 2 - (int)dialog_extents.width / 2 - DIALOG_PADDING;
    int width = (int)dialog_extents.width + DIALOG_PADDING * 2;
    int height = (int)dialog_extents.height + DIALOG_PADDING * 2;
    if (show_ok) {
        /* Make room for the ok button */
        cairo_text_extents(table->buffer_context, "OK", &ok_extents);
        height += (int)ok_extents.height + DIALOG_PADDING * 3;
    }
    gcribbage_table_render_rounded_rectangle(
        table,
        x,
        y,
        width,
        height,
        5, // radius
        0.13, // r
        0.33, // g
        0.21 // b
    ); 

    x = win_width / 2 - (int)dialog_extents.width / 2;
    y += (int)dialog_extents.height + DIALOG_PADDING;
    cairo_set_source_rgb(table->buffer_context, 0.8, 0.8, 0.8);
    cairo_move_to(table->buffer_context, x, y);
    cairo_show_text(table->buffer_context, text);

    if (show_ok) {
        /* Extents will have OK text extents at this point */
        x = win_width / 2 - (int)ok_extents.width / 2 - DIALOG_PADDING;
        y += DIALOG_PADDING;
        width = (int)ok_extents.width + DIALOG_PADDING * 2;
        height = (int)ok_extents.height + DIALOG_PADDING * 2;
        gcribbage_table_render_rounded_rectangle(
            table,
            x,
            y,
            width,
            height,
            5, // radius
            0.21, // r
            0.59, // g
            0.37 // b
        );
        gcribbage_table_add_hit_box(table, x, y, width, height, 0);
        x = win_width / 2 - (int)ok_extents.width / 2;
        y += (int)ok_extents.height + DIALOG_PADDING;
        cairo_set_source_rgb(table->buffer_context, 0.8, 0.8, 0.8);
        cairo_move_to(table->buffer_context, x, y);
        cairo_show_text(table->buffer_context, "OK");
    }
}

void gcribbage_table_render_choose_dealer(GCribbageTable *table, struct RenderDeckCutScene *scene) {
    int width = table->card_options.fan_spacing * CARD_MAX_CUT_POSITIONS + table->card_options.width;
    cairo_surface_t *surface = cairo_get_target(table->buffer_context);
    int win_width = cairo_image_surface_get_width(surface);

    width = win_width / 2 - width / 2;

    // Reset hitbox count to 0
    table->num_hitboxes = 0;

    int hit_box_width = 0;
    for (int i = 0; i < CARD_MAX_CUT_POSITIONS; i++) {
        if (i == scene->chosen_slots[0] || i == scene->chosen_slots[1]) {
            hit_box_width += table->card_options.fan_spacing;
        } else {
            gcribbage_table_render_card_back(
                table,
                width + i * table->card_options.fan_spacing,
                table->card_options.top_offset
            );
            if (i < CARD_MAX_CUT_POSITIONS - 1) {
                gcribbage_table_add_hit_box(
                    table,
                    width + i * table->card_options.fan_spacing,
                    table->card_options.top_offset,
                    hit_box_width + table->card_options.fan_spacing,
                    table->card_options.height,
                    i + 1
                );
            } else {
                gcribbage_table_add_hit_box(
                    table,
                    width + i * table->card_options.fan_spacing,
                    table->card_options.top_offset,
                    table->card_options.width,
                    table->card_options.height,
                    i + 1
                );
            }
            hit_box_width = 0;
        }
    }

    if (scene->player_card != CARD_NONE) {
        gcribbage_table_render_card(
            table,
            scene->player_card,
            width,
            table->card_options.middle_offset
        );

        gcribbage_table_render_card(
            table,
            scene->cpu_card,
            width + table->card_options.fan_spacing * 12,
            table->card_options.middle_offset
        );

        if ((scene->player_card & 0xf) < (scene->cpu_card & 0xf)) {
            gcribbage_table_render_dialog(
                table,
                "You deal first.",
                table->card_options.middle_offset,
                1
            );
        } else {
            gcribbage_table_render_dialog(
                table,
                "CPU deals first.",
                table->card_options.middle_offset,
                1
            );
        }
    } else {
        gcribbage_table_render_dialog(
            table,
            "Choose a card. Lowest card deals first.",
            table->card_options.middle_offset,
            0
        );
    }
}

void gcribbage_table_render_choose_crib(GCribbageTable *table, struct ChooseCribScene *scene) {
    int width = table->card_options.fan_spacing * 5 + table->card_options.width;
    cairo_surface_t *surface = cairo_get_target(table->buffer_context);
    int win_width = cairo_image_surface_get_width(surface);
    int middle = win_width / 2;
    width = middle - width / 2;

    for (int i = 0; i < 6; i++){
        gcribbage_table_render_card_back(
            table,
            width + i * table->card_options.fan_spacing,
            table->card_options.top_offset
        );
        gcribbage_table_render_card(
            table,
            scene->player_cards[i],
            width + i * table->card_options.fan_spacing,
            table->card_options.bottom_offset
        );
        gcribbage_table_add_hit_box(
            table,
            width + i * table->card_options.fan_spacing,
            table->card_options.bottom_offset,
            table->card_options.width,
            table->card_options.height,
            POSITION_NONE
        );
    }

    gcribbage_table_render_card_back (
        table,
        middle - (table->card_options.width + table->card_options.fan_spacing) * 2,
        table->card_options.middle_offset
    );

    gcribbage_table_render_dialog(
        table,
        "Choose two cards for the crib.",
        table->card_options.middle_offset,
        0
    );


}

void render_buffer(GCribbageTable *table) {
    struct RenderScene scene;
    game_data_get_render_scene(table->game_data, &scene);
    clear_buffer(table->buffer_context);
    table->num_hitboxes = 0;
    switch (scene.type) {
        case DECK_CUT_SCENE:
            gcribbage_table_render_choose_dealer(
                table,
                &scene.deck_cut_scene
            );
            break;
        case CHOOSE_CRIB_SCENE:
            gcribbage_table_render_choose_crib(table, &scene.choose_crib_scene);
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
    g_log(G_LOG_DOMAIN, G_LOG_LEVEL_DEBUG, "Advancing game, position %d", player_position_choice);
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
            gcribbage_table_advance_game(table, hitbox->advance_data);
            break;
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
