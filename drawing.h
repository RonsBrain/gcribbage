#pragma once
#include "hitbox.h"
#include "simulation.h"
#include <cairo.h>
#include <gtk/gtk.h>

void draw_clear_buffer(cairo_t *renderer, GdkRGBA *color);
void draw_card(cairo_t *renderer, GdkPixbuf *card_images, struct Card card,
               int x, int y, int width, int height,
               struct HitboxList *hitbox_list, int hitbox_data);
void draw_card_back(cairo_t *renderer, GdkPixbuf *back_image, int x, int y,
                    int width, int height, struct HitboxList *hitbox_list,
                    int hitbox_data);
void draw_dialog(cairo_t *renderer, char *text, struct HitboxList *hitbox_list,
                 int midpoint, int y, int padding, int hitbox_data);
void draw_text(cairo_t *renderer, char *text, int x, int y, int padding);
void draw_scores(cairo_t *renderer, int scores[PLAYER_END], int x, int y,
                 int padding);
