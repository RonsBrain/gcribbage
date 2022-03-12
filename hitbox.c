#include "hitbox.h"
#include <stddef.h>
#include <stdlib.h>

void hitbox_list_add_hitbox(struct HitboxList *hitbox_list, int x, int y,
                            int width, int height, int data) {
  if (hitbox_list->num_hitboxes >= MAX_HITBOXES) {
    abort();
  }
  struct Hitbox *hitbox = &hitbox_list->hitboxes[hitbox_list->num_hitboxes];
  hitbox->x = x;
  hitbox->y = y;
  hitbox->width = width;
  hitbox->height = height;
  hitbox->data = data;

  hitbox_list->num_hitboxes += 1;
}

void hitbox_list_clear(struct HitboxList *hitbox_list) {
  hitbox_list->num_hitboxes = 0;
}

int hitbox_list_hit_data(struct HitboxList *hitbox_list, int x, int y) {
  struct Hitbox *current;
  int result = HITBOX_NO_HIT;

  /* The scene renderers all lay out the cards from left to right,
   * and so the hitboxes are all arranged left to right. Since cards can
   * overlap, we want to choose the rightmost card that was touched. This
   * is a messy implementation detail, but it keeps us from having to build
   * in some sort of z-index for the hit boxes.
   */
  for (int i = 0; i < hitbox_list->num_hitboxes; i++) {
    current = &hitbox_list->hitboxes[i];
    if (x >= current->x && x < current->x + current->width && y >= current->y &&
        y < current->y + current->height) {
      result = current->data;
    }
  }
  return result;
}
