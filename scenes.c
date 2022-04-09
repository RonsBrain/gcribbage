#include "scenes.h"
#include "drawing.h"
#include "hitbox.h"
#include "simulation.h"
#include <cairo.h>

#define CARD_MAX_CUT_POSITIONS 13

void scene_choose_dealer(cairo_t *renderer, struct RenderDeckCutScene *scene,
                         struct HitboxList *hitbox_list,
                         struct LayoutOptions *layout_options) {
  int width = layout_options->fan_spacing * CARD_MAX_CUT_POSITIONS +
              layout_options->card_width;
  cairo_surface_t *surface = cairo_get_target(renderer);
  int win_width = cairo_image_surface_get_width(surface);
  width = win_width / 2 - width / 2;
  for (int i = 0; i < CARD_MAX_CUT_POSITIONS; i++) {
    if (scene->chosen_slot != i + 1) {
      int position = POSITION_NONE;
      struct HitboxList *possible_list = NULL;
      if (IS_SAME_CARD(scene->human_card, CARD_NONE)) {
        /* Human has not yet chosen a card, so we want them to be able to. */
        position = i + 1;
        possible_list = hitbox_list;
      }
      draw_card_back(renderer, layout_options->images.card_back,
                     width + i * layout_options->fan_spacing,
                     layout_options->top_offset, layout_options->card_width,
                     layout_options->card_height, possible_list, position);
    }
  }
  if (!IS_SAME_CARD(scene->human_card, CARD_NONE)) {
    draw_card(renderer, layout_options->images.card_images, scene->human_card,
              width, layout_options->middle_offset, layout_options->card_width,
              layout_options->card_height, NULL, 0);
  } else {
    draw_dialog(renderer, "Choose a card. Lowest card deals first.", NULL,
                win_width / 2, layout_options->middle_offset,
                layout_options->padding, 0);
  }
}

void scene_announce_dealer(cairo_t *renderer, struct AnnounceDealerScene *scene,
                           struct HitboxList *hitbox_list,
                           struct LayoutOptions *layout_options) {
  int width = layout_options->fan_spacing * CARD_MAX_CUT_POSITIONS +
              layout_options->card_width;
  cairo_surface_t *surface = cairo_get_target(renderer);
  int win_width = cairo_image_surface_get_width(surface);
  width = win_width / 2 - width / 2;

  for (int i = 0; i < CARD_MAX_CUT_POSITIONS; i++) {
    if (scene->chosen_slots[0] != i + 1 && scene->chosen_slots[1] != i) {
      draw_card_back(renderer, layout_options->images.card_back,
                     width + i * layout_options->fan_spacing,
                     layout_options->top_offset, layout_options->card_width,
                     layout_options->card_height, NULL, 0);
    }
  }

  draw_card(renderer, layout_options->images.card_images,
            scene->chosen_cards[PLAYER_HUMAN], width,
            layout_options->middle_offset, layout_options->card_width,
            layout_options->card_height, NULL, 0);

  draw_card(renderer, layout_options->images.card_images,
            scene->chosen_cards[PLAYER_CPU],
            width + layout_options->fan_spacing * 12,
            layout_options->middle_offset, layout_options->card_width,
            layout_options->card_height, NULL, 0);

  char *text;
  if (scene->first_dealer == PLAYER_HUMAN) {
    text = "You deal first.";
  } else {
    text = "CPU deals first.";
  }
  draw_dialog(renderer, text, hitbox_list, win_width / 2,
              layout_options->middle_offset, layout_options->padding, 1);
}

void scene_choose_crib(cairo_t *renderer, struct ChooseCribScene *scene,
                       struct HitboxList *hitbox_list,
                       struct LayoutOptions *layout_options) {
  int width = layout_options->fan_spacing * 5 + layout_options->card_width;
  cairo_surface_t *surface = cairo_get_target(renderer);
  int win_width = cairo_image_surface_get_width(surface);
  int middle = win_width / 2;
  width = middle - width / 2;

  draw_scores(renderer, scene->scores, middle, layout_options->score_offset,
              layout_options->padding);

  for (int i = 0; i < 6; i++) {
    draw_card_back(renderer, layout_options->images.card_back,
                   width + i * layout_options->fan_spacing,
                   layout_options->top_offset, layout_options->card_width,
                   layout_options->card_height, NULL, 0);
    int offset = 0;
    if (i + 1 == scene->human_crib_choices[0] ||
        i + 1 == scene->human_crib_choices[1]) {
      /* This card is chosen, so draw it offset slightly */
      offset = layout_options->fan_spacing;
    }
    if (!scene->ready_to_proceed || (scene->ready_to_proceed && offset != 0)) {
      /* If the human has not selected enough cards, we want them to be
       * able to select any of them.
       *
       * If the human has selected two cards, we want them to be able to
       * back out but we don't want them to select more cards.
       */
      draw_card(renderer, layout_options->images.card_images,
                scene->human_cards[i], width + i * layout_options->fan_spacing,
                layout_options->bottom_offset - offset,
                layout_options->card_width, layout_options->card_height,
                hitbox_list, i + 1);
    } else {
      draw_card(renderer, layout_options->images.card_images,
                scene->human_cards[i], width + i * layout_options->fan_spacing,
                layout_options->bottom_offset - offset,
                layout_options->card_width, layout_options->card_height, NULL,
                0);
    }
  }

  draw_card_back(
      renderer, layout_options->images.card_back,
      middle - (layout_options->card_width + layout_options->fan_spacing) * 2,
      layout_options->middle_offset, layout_options->card_width,
      layout_options->card_height, NULL, 0);

  int crib_x, crib_y;
  char *crib_text;

  crib_x = win_width / 2 +
           (layout_options->fan_spacing * 5 + layout_options->fan_spacing +
            layout_options->card_width) /
               2 +
           layout_options->padding;

  if (scene->crib_player == PLAYER_HUMAN) {
    crib_text = "Your crib.";
    crib_y = layout_options->bottom_offset;
  } else {
    crib_text = "CPU's crib.";
    crib_y = layout_options->top_offset;
  }

  draw_text(renderer, crib_text, crib_x, crib_y, layout_options->padding);

  if (!scene->ready_to_proceed) {
    draw_dialog(renderer, "Choose two cards for the crib.", NULL, win_width / 2,
                layout_options->middle_offset, layout_options->padding, 0);
  } else {
    draw_dialog(renderer, "Are these the crib cards?", hitbox_list,
                win_width / 2, layout_options->middle_offset,
                layout_options->padding, 0);
  }
}

void scene_announce_nibs(cairo_t *renderer, struct AnnounceNibsScene *scene,
                         struct HitboxList *hitbox_list,
                         struct LayoutOptions *layout_options) {
  int card_width = layout_options->fan_spacing * 3 + layout_options->card_width;
  cairo_surface_t *surface = cairo_get_target(renderer);
  int win_width = cairo_image_surface_get_width(surface);
  int middle = win_width / 2;
  int width = middle - card_width / 2;

  draw_scores(renderer, scene->scores, middle, layout_options->score_offset,
              layout_options->padding);

  for (int i = 0; i < 4; i++) {
    draw_card_back(renderer, layout_options->images.card_back,
                   width + i * layout_options->fan_spacing,
                   layout_options->top_offset, layout_options->card_width,
                   layout_options->card_height, NULL, 0);
    draw_card(renderer, layout_options->images.card_images,
              scene->human_cards[i], width + i * layout_options->fan_spacing,
              layout_options->bottom_offset, layout_options->card_width,
              layout_options->card_height, NULL, 0);
  }

  draw_card(renderer, layout_options->images.card_images, scene->up_card,
            middle -
                (layout_options->card_width + layout_options->fan_spacing) * 2,
            layout_options->middle_offset, layout_options->card_width,
            layout_options->card_height, NULL, 0);

  int crib_x, crib_y;
  char *text;

  crib_x =
      win_width / 2 + card_width / 2 + card_width + layout_options->padding;

  if (scene->dealer == PLAYER_HUMAN) {
    crib_y = layout_options->bottom_offset;
    text = "You get nibs for 2 points!";
  } else {
    crib_y = layout_options->top_offset;
    text = "CPU gets nibs for 2 points.";
  }

  draw_card_back(renderer, layout_options->images.card_back, crib_x, crib_y,
                 layout_options->card_width, layout_options->card_height, NULL,
                 0);
  draw_dialog(renderer, text, hitbox_list, win_width / 2,
              layout_options->middle_offset, layout_options->padding, 0);
}

void scene_pegging(cairo_t *renderer, struct PeggingScene *scene,
                   struct HitboxList *hitbox_list,
                   struct LayoutOptions *layout_options) {
  int card_width = layout_options->fan_spacing * 3 + layout_options->card_width;
  cairo_surface_t *surface = cairo_get_target(renderer);
  int win_width = cairo_image_surface_get_width(surface);
  int middle = win_width / 2;
  int width = middle - card_width / 2;

  draw_scores(renderer, scene->scores, middle, layout_options->score_offset,
              layout_options->padding);

  for (int i = 0; i < scene->remaining_cpu_cards; i++) {
    draw_card_back(renderer, layout_options->images.card_back,
                   width + i * layout_options->fan_spacing,
                   layout_options->top_offset, layout_options->card_width,
                   layout_options->card_height, NULL, 0);
  }
  struct HitboxList *hitbox_to_use =
      scene->current_player == PLAYER_HUMAN ? hitbox_list : NULL;
  for (int i = 0; i < 4; i++) {
    if (IS_CARD(scene->human_cards[i])) {
      draw_card(renderer, layout_options->images.card_images,
                scene->human_cards[i], width + i * layout_options->fan_spacing,
                layout_options->bottom_offset, layout_options->card_width,
                layout_options->card_height, hitbox_to_use, i + 1);
    }
  }

  draw_card(renderer, layout_options->images.card_images, scene->up_card,
            middle -
                (layout_options->card_width + layout_options->fan_spacing) * 2,
            layout_options->middle_offset, layout_options->card_width,
            layout_options->card_height, NULL, 0);

  int crib_x, crib_y;

  crib_x =
      win_width / 2 + card_width / 2 + card_width + layout_options->padding;

  if (scene->dealer == PLAYER_HUMAN) {
    crib_y = layout_options->bottom_offset;
  } else {
    crib_y = layout_options->top_offset;
  }

  draw_card_back(renderer, layout_options->images.card_back, crib_x, crib_y,
                 layout_options->card_width, layout_options->card_height, NULL,
                 0);

  int play_x =
      win_width / 2 -
      (layout_options->fan_spacing * 7 + layout_options->card_width) / 2;
  struct Card *played_card = scene->played_cards;
  while (IS_CARD((*played_card))) {
    draw_card(renderer, layout_options->images.card_images, *played_card,
              play_x, layout_options->middle_offset, layout_options->card_width,
              layout_options->card_height, NULL, 0);
    play_x += layout_options->fan_spacing;
    played_card++;
  }
  play_x += layout_options->card_width + layout_options->padding;
  char score_text[3];
  int written = snprintf(score_text, 3, "%d", scene->pegging_count);
  if (written > 3) {
    abort();
  }
  draw_text(renderer, score_text, play_x, layout_options->middle_offset,
            layout_options->padding);

  if (scene->called_go[PLAYER_HUMAN]) {
    draw_text(renderer, "Go",
              win_width / 2 - card_width / 2 - card_width -
                  layout_options->padding,
              layout_options->bottom_offset, layout_options->padding);
  }

  if (scene->called_go[PLAYER_CPU]) {
    draw_text(renderer, "Go",
              win_width / 2 - card_width / 2 - card_width -
                  layout_options->padding,
              layout_options->top_offset, layout_options->padding);
  }

  if (scene->last_card) {
    char *last_card_text;
    if (scene->last_card_player == PLAYER_HUMAN) {
      last_card_text = "You played last card for 1 point.";
    } else {
      last_card_text = "CPU played last card for 1 point.";
    }
    draw_dialog(renderer, last_card_text, hitbox_list, win_width / 2,
                layout_options->middle_offset, layout_options->padding, 0);
  }
}
