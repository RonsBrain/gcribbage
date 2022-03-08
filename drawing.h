#pragma once
#include <cairo.h>
#include <gtk/gtk.h>
#include "hitbox.h"

void draw_clear_buffer(cairo_t *renderer, GdkRGBA *color);
void draw_card(cairo_t *renderer, GdkPixbuf *card_images, char card, int x, int y, int width, int height);
void draw_card_back(cairo_t *renderer, GdkPixbuf *back_image, int x, int y, int width, int height);
void draw_dialog(cairo_t *renderer, char *text, struct HitboxList *hitbox_list, int midpoint, int y, int padding, int hitbox_data);
