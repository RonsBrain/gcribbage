#include <cairo.h>
#include <gtk/gtk.h>
#include "drawing.h"
#include "hitbox.h"
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
    int padding;
};

struct _GCribbageTable {
    GtkWidget parent;
    struct GameData *game_data;
    cairo_t *buffer_context;
    GdkPixbuf *card_images;
    GdkPixbuf *card_back;
    struct HitboxList hitbox_list;
    int num_hitboxes;
    struct CardOptions card_options;
};

G_DEFINE_TYPE(GCribbageTable, gcribbage_table, GTK_TYPE_DRAWING_AREA);

GdkRGBA BACKGROUND_COLOR = { 0.f, 0.6f, 0.f, 1.f };

void clear_buffer(cairo_t *cr) {
    gdk_cairo_set_source_rgba(cr, &BACKGROUND_COLOR);
    cairo_paint(cr);
}

void gcribbage_table_render_choose_dealer(GCribbageTable *table, struct RenderDeckCutScene *scene) {
    int width = table->card_options.fan_spacing * CARD_MAX_CUT_POSITIONS + table->card_options.width;
    cairo_surface_t *surface = cairo_get_target(table->buffer_context);
    int win_width = cairo_image_surface_get_width(surface);
    hitbox_list_clear(&table->hitbox_list);
    width = win_width / 2 - width / 2;

    int hit_box_width = 0;
    for (int i = 0; i < CARD_MAX_CUT_POSITIONS; i++) {
        if (i == scene->chosen_slots[0] || i == scene->chosen_slots[1]) {
            hit_box_width += table->card_options.fan_spacing;
        } else {
            draw_card_back(
                table->buffer_context,
                table->card_back,
                width + i * table->card_options.fan_spacing,
                table->card_options.top_offset,
                table->card_options.width,
                table->card_options.height
            );
            if (i < CARD_MAX_CUT_POSITIONS - 1) {
                hitbox_list_add_hitbox(
                    &table->hitbox_list,
                    width + i * table->card_options.fan_spacing,
                    table->card_options.top_offset,
                    hit_box_width + table->card_options.fan_spacing,
                    table->card_options.height,
                    i + 1
                );
            } else {
                hitbox_list_add_hitbox(
                    &table->hitbox_list,
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
        draw_card(
            table->buffer_context,
            table->card_images,
            scene->player_card,
            width,
            table->card_options.middle_offset,
            table->card_options.width,
            table->card_options.height
        );

        draw_card(
            table->buffer_context,
            table->card_images,
            scene->cpu_card,
            width + table->card_options.fan_spacing * 12,
            table->card_options.middle_offset,
            table->card_options.width,
            table->card_options.height
        );

        if ((scene->player_card & 0xf) < (scene->cpu_card & 0xf)) {
            draw_dialog(
                table->buffer_context,
                "You deal first.",
                &table->hitbox_list,
                win_width / 2,
                table->card_options.middle_offset,
                table->card_options.padding,
                0
            );
        } else {
            draw_dialog(
                table->buffer_context,
                "CPU deals first.",
                &table->hitbox_list,
                win_width / 2,
                table->card_options.middle_offset,
                table->card_options.padding,
                0
            );
        }
    } else {
        draw_dialog(
            table->buffer_context,
            "Choose a card. Lowest card deals first.",
            NULL,
            win_width / 2,
            table->card_options.middle_offset,
            table->card_options.padding,
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
        draw_card_back(
            table->buffer_context,
            table->card_back,
            width + i * table->card_options.fan_spacing,
            table->card_options.top_offset,
            table->card_options.width,
            table->card_options.height
        );
        draw_card(
            table->buffer_context,
            table->card_images,
            scene->player_cards[i],
            width + i * table->card_options.fan_spacing,
            table->card_options.bottom_offset,
            table->card_options.width,
            table->card_options.height
        );
        hitbox_list_add_hitbox(
            &table->hitbox_list,
            width + i * table->card_options.fan_spacing,
            table->card_options.bottom_offset,
            table->card_options.width,
            table->card_options.height,
            POSITION_NONE
        );
    }

    draw_card_back (
        table->buffer_context,
        table->card_back,
        middle - (table->card_options.width + table->card_options.fan_spacing) * 2,
        table->card_options.middle_offset,
        table->card_options.width,
        table->card_options.height
    );

    draw_dialog(
        table->buffer_context,
        "Choose two cards for the crib.",
        NULL,
        win_width / 2,
        table->card_options.middle_offset,
        table->card_options.padding,
        0
    );


}

void render_buffer(GCribbageTable *table) {
    struct RenderScene scene;
    game_data_get_render_scene(table->game_data, &scene);
    draw_clear_buffer(table->buffer_context, &BACKGROUND_COLOR);
    hitbox_list_clear(&table->hitbox_list);
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
    struct Hitbox *hitbox;
    GCribbageTable *table = GCRIBBAGE_TABLE(area);
    hitbox = hitbox_list_intersection(&table->hitbox_list, (int)x, (int)y);
    if (hitbox) {
        gcribbage_table_advance_game(table, hitbox->data);
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
    table->card_options.padding = 5;
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
