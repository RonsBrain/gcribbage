#include "hitbox.h"
#include "simulation.h"
#include <cairo.h>
#include <gtk/gtk.h>

struct Images {
  GdkPixbuf *card_images;
  GdkPixbuf *card_back;
};

struct LayoutOptions {
  int card_width;
  int card_height;
  int fan_spacing;
  int top_offset;
  int middle_offset;
  int bottom_offset;
  int score_offset;
  int padding;
  struct Images images;
};

void scene_choose_dealer(cairo_t *renderer, struct RenderDeckCutScene *scene,
                         struct HitboxList *hitbox_list,
                         struct LayoutOptions *layout_options);

void scene_announce_dealer(cairo_t *renderer, struct AnnounceDealerScene *scene,
                           struct HitboxList *hitbox_list,
                           struct LayoutOptions *layout_options);

void scene_choose_crib(cairo_t *renderer, struct ChooseCribScene *scene,
                       struct HitboxList *hitbox_list,
                       struct LayoutOptions *layout_options);

void scene_announce_nibs(cairo_t *renderer, struct AnnounceNibsScene *scene,
                         struct HitboxList *hitbox_list,
                         struct LayoutOptions *layout_options);
