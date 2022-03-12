#pragma once
#define MAX_HITBOXES 13
#define HITBOX_NO_HIT -1

struct Hitbox {
  int x;
  int y;
  int width;
  int height;
  int data;
};

struct HitboxList {
  struct Hitbox hitboxes[MAX_HITBOXES];
  int num_hitboxes;
};

void hitbox_list_add_hitbox(struct HitboxList *hitbox_list, int x, int y,
                            int width, int height, int data);
void hitbox_list_clear(struct HitboxList *hitbox_list);
int hitbox_list_hit_data(struct HitboxList *hitbox_list, int x, int y);
