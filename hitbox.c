#include <stddef.h>
#include <stdlib.h>
#include "hitbox.h"

void hitbox_list_add_hitbox(struct HitboxList *hitbox_list, int x, int y, int width, int height, int data) {
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

struct Hitbox *hitbox_list_intersection(struct HitboxList *hitbox_list, int x, int y) {
    struct Hitbox *hitbox;
    for (int i = 0; i < hitbox_list->num_hitboxes; i++) {
        hitbox = &hitbox_list->hitboxes[i];
        if (
            x >= hitbox->x && x < hitbox->x + hitbox->width &&
            y >= hitbox->y && y < hitbox->y + hitbox->height
        ) {
            return hitbox;
        }
    }
    return NULL;
}
