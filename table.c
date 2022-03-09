#include "table.h"
#include "drawing.h"
#include "hitbox.h"
#include "scenes.h"
#include "simulation.h"
#include <cairo.h>
#include <gtk/gtk.h>

#define MAX_HITBOXES 13
#define CARD_MAX_CUT_POSITIONS 13

struct _GCribbageTable {
  GtkWidget parent;
  struct GameData *game_data;
  cairo_t *buffer_context;
  struct HitboxList hitbox_list;
  struct LayoutOptions layout_options;
};

G_DEFINE_TYPE(GCribbageTable, gcribbage_table, GTK_TYPE_DRAWING_AREA);

GdkRGBA BACKGROUND_COLOR = {0.f, 0.6f, 0.f, 1.f};

void clear_buffer(cairo_t *cr) {
  gdk_cairo_set_source_rgba(cr, &BACKGROUND_COLOR);
  cairo_paint(cr);
}

void render_buffer(GCribbageTable *table) {
  struct RenderScene scene;
  game_data_get_render_scene(table->game_data, &scene);
  draw_clear_buffer(table->buffer_context, &BACKGROUND_COLOR);
  hitbox_list_clear(&table->hitbox_list);
  switch (scene.type) {
  case DECK_CUT_SCENE:
    scene_choose_dealer(table->buffer_context, &scene.deck_cut_scene,
                        &table->hitbox_list, &table->layout_options);
    break;
  case CHOOSE_CRIB_SCENE:
    scene_choose_crib(table->buffer_context, &scene.choose_crib_scene,
                      &table->hitbox_list, &table->layout_options);
    break;
  default:
    break;
  }
}

void gcribbage_table_handle_resize(GCribbageTable *table, int width, int height,
                                   gpointer userdata) {
  if (table->buffer_context) {
    cairo_destroy(table->buffer_context);
  }
  cairo_surface_t *surface =
      cairo_image_surface_create(CAIRO_FORMAT_ARGB32, width, height);
  table->buffer_context = cairo_create(surface);
  cairo_select_font_face(table->buffer_context, "sans-serif",
                         CAIRO_FONT_SLANT_NORMAL, CAIRO_FONT_WEIGHT_NORMAL);
  cairo_set_font_size(table->buffer_context, 18);
  table->layout_options.top_offset = 25;
  table->layout_options.middle_offset =
      height / 2 - table->layout_options.card_height / 2;
  table->layout_options.bottom_offset =
      height - 25 - table->layout_options.card_height;
  render_buffer(table);
}

static void draw(GtkDrawingArea *table, cairo_t *cr, int width, int height,
                 gpointer data) {
  cairo_set_source_surface(
      cr, cairo_get_target(GCRIBBAGE_TABLE(table)->buffer_context), 0, 0);
  cairo_paint(cr);
}

static void gcribbage_table_advance_game(GCribbageTable *table,
                                         int player_position_choice) {
  game_data_advance_game(table->game_data, player_position_choice);
  render_buffer(table);
  gtk_widget_queue_draw(GTK_WIDGET(table));
}

static void pressed(GtkGestureClick *gesture, int n_press, double x, double y,
                    GtkWidget *area) {
  struct Hitbox *hitbox;
  GCribbageTable *table = GCRIBBAGE_TABLE(area);
  hitbox = hitbox_list_intersection(&table->hitbox_list, (int)x, (int)y);
  if (hitbox) {
    gcribbage_table_advance_game(table, hitbox->data);
  }
}

void gcribbage_table_start_new_game(GCribbageTable *table) {
  game_data_destroy(table->game_data);
  table->game_data = game_data_create();
  render_buffer(table);
  gtk_widget_queue_draw(GTK_WIDGET(table));
}

static void gcribbage_table_init(GCribbageTable *table) {
  GtkGesture *press;
  press = gtk_gesture_click_new();
  gtk_gesture_single_set_button(GTK_GESTURE_SINGLE(press), GDK_BUTTON_PRIMARY);
  gtk_widget_add_controller(GTK_WIDGET(table), GTK_EVENT_CONTROLLER(press));
  g_signal_connect(press, "pressed", G_CALLBACK(pressed), table);
  g_signal_connect(table, "resize", G_CALLBACK(gcribbage_table_handle_resize),
                   NULL);
  GError *error = NULL;
  table->layout_options.card_images = gdk_pixbuf_new_from_resource(
      "/com/ronsbrain/gcribbage/assets/cards.png", &error);
  table->layout_options.card_back = gdk_pixbuf_new_from_resource(
      "/com/ronsbrain/gcribbage/assets/back.png", &error);
  table->layout_options.card_width =
      gdk_pixbuf_get_width(table->layout_options.card_images) / 13;
  table->layout_options.card_height =
      gdk_pixbuf_get_height(table->layout_options.card_images) / 4;
  table->layout_options.fan_spacing = 30;
  table->layout_options.padding = 5;
  gtk_drawing_area_set_draw_func(GTK_DRAWING_AREA(table), draw, (gpointer)table,
                                 NULL);
  table->game_data = game_data_create();
}

static void gcribbage_table_class_init(GCribbageTableClass *klass) {}
