#include "drawing.h"
#include <stdio.h>

void draw_clear_buffer(cairo_t *renderer, GdkRGBA *color) {
  gdk_cairo_set_source_rgba(renderer, color);
  cairo_paint(renderer);
}

void draw_card(cairo_t *renderer, GdkPixbuf *card_images, char card, int x,
               int y, int width, int height) {
  int rank =
      (card & 0xf) - 1; // Rank goes from 1 - 13 but we want to offset from 0
  int suit = (card >> 4);
  int src_x = width * rank;
  int src_y = height * suit;
  gdk_cairo_set_source_pixbuf(renderer, card_images, x - src_x, y - src_y);
  cairo_rectangle(renderer, x, y, width, height);
  cairo_fill(renderer);
}

void draw_card_back(cairo_t *renderer, GdkPixbuf *back_image, int x, int y,
                    int width, int height) {
  gdk_cairo_set_source_pixbuf(renderer, back_image, x, y);
  cairo_rectangle(renderer, x, y, width, height);
  cairo_fill(renderer);
}

#define PI 3.14159265358979323846
void draw_rounded_rectangle(cairo_t *renderer, double x, double y, double width,
                            double height, double radius, double r, double g,
                            double b) {
  cairo_new_sub_path(renderer);

  cairo_arc(renderer, x + radius, y + radius, radius, PI, 3 * PI / 2);
  cairo_arc(renderer, x + width - radius, y + radius, radius, 3 * PI / 2,
            2 * PI);
  cairo_arc(renderer, x + width - radius, y + height - radius, radius, 0,
            PI / 2);
  cairo_arc(renderer, x + radius, y + height - radius, radius, PI / 2, PI);
  cairo_close_path(renderer);

  cairo_set_source_rgb(renderer, r, g, b);
  cairo_fill(renderer);
}

void draw_text(cairo_t *renderer, char *text, int x, int y, int padding) {
  cairo_text_extents_t text_extents;
  cairo_text_extents(renderer, text, &text_extents);

  draw_rounded_rectangle(renderer, x, y, (int)text_extents.width + padding * 2,
                         (int)text_extents.height + padding * 2, 5, 0.13, 0.33,
                         0.21);
  x += padding;
  y += padding + (int)text_extents.height;
  cairo_set_source_rgb(renderer, 0.8, 0.8, 0.8);
  cairo_move_to(renderer, x, y);
  cairo_show_text(renderer, text);
}

void draw_dialog(cairo_t *renderer, char *text, struct HitboxList *hitbox_list,
                 int midpoint, int y, int padding, int hitbox_data) {
  cairo_surface_t *surface = cairo_get_target(renderer);
  int win_width = cairo_image_surface_get_width(surface);
  cairo_text_extents_t text_extents, ok_extents;
  cairo_text_extents(renderer, text, &text_extents);
  int x = win_width / 2 - (int)text_extents.width / 2 - padding;
  int width = (int)text_extents.width + padding * 2;
  int height = (int)text_extents.height + padding * 2;
  if (hitbox_list) {
    /* Make room for the ok button */
    cairo_text_extents(renderer, "OK", &ok_extents);
    height += (int)ok_extents.height + padding * 3;
  }
  draw_rounded_rectangle(renderer, x, y, width, height,
                         5,    // radius
                         0.13, // r
                         0.33, // g
                         0.21  // b
  );

  x = win_width / 2 - (int)text_extents.width / 2;
  y += (int)text_extents.height + padding;
  cairo_set_source_rgb(renderer, 0.8, 0.8, 0.8);
  cairo_move_to(renderer, x, y);
  cairo_show_text(renderer, text);
  if (hitbox_list) {
    /* Extents will have OK text extents at this point */
    x = win_width / 2 - (int)ok_extents.width / 2 - padding;
    y += padding;
    width = (int)ok_extents.width + padding * 2;
    height = (int)ok_extents.height + padding * 2;
    draw_rounded_rectangle(renderer, x, y, width, height,
                           5,    // radius
                           0.21, // r
                           0.59, // g
                           0.37  // b
    );
    hitbox_list_add_hitbox(hitbox_list, x, y, width, height, hitbox_data);
    x = win_width / 2 - (int)ok_extents.width / 2;
    y += (int)ok_extents.height + padding;
    cairo_set_source_rgb(renderer, 0.8, 0.8, 0.8);
    cairo_move_to(renderer, x, y);
    cairo_show_text(renderer, "OK");
  }
}

void draw_scores(cairo_t *renderer, int scores[PLAYER_END], int middle, int y,
                 int padding) {
  char score_buffer[9]; // Max 3 characters plus null terminator
  cairo_text_extents_t text_extents;
  char *format;
  for (int i = PLAYER_HUMAN; i < PLAYER_END; i++) {
    if (i == PLAYER_HUMAN) {
      format = "You: %d";
    } else {
      format = "CPU: %d";
    }
    int result = snprintf(score_buffer, 9, format, scores[i]);
    if (result > 9 || result < 0) {
      g_log(G_LOG_DOMAIN, G_LOG_LEVEL_CRITICAL,
            "snprintf returned %d for score %d", result, scores[i]);
      abort();
    }
    cairo_text_extents(renderer, score_buffer, &text_extents);
    int this_x = middle - 121 - padding;
    int this_y;
    if (i == PLAYER_HUMAN) {
      this_y = y;
    } else {
      this_y = y + (int)text_extents.height + padding * 3;
    }
    draw_rounded_rectangle(renderer, this_x, this_y, 242 + padding * 2,
                           (int)text_extents.height + padding * 2, 5, 0.13,
                           0.33, 0.21);
    draw_rounded_rectangle(
        renderer, this_x, this_y, scores[i] * 2 + padding * 2,
        (int)text_extents.height + padding * 2, 5, 0.13, 0.13, 0.60);
    cairo_set_source_rgb(renderer, 0.8, 0.8, 0.8);
    cairo_move_to(renderer,
                  this_x + padding + 121 - (int)(text_extents.width / 2.f),
                  this_y + (int)text_extents.height + padding);
    cairo_show_text(renderer, score_buffer);
  }
}
