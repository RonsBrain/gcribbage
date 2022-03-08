#include <cairo.h>
#include "drawing.h"
#include "hitbox.h"
#include "scenes.h"
#include "simulation.h"

#define CARD_MAX_CUT_POSITIONS 13
void scene_choose_dealer(cairo_t *renderer, struct RenderDeckCutScene *scene, struct HitboxList *hitbox_list, struct LayoutOptions *layout_options) {
    int width = layout_options->fan_spacing * CARD_MAX_CUT_POSITIONS + layout_options->card_width;
    cairo_surface_t *surface = cairo_get_target(renderer);
    int win_width = cairo_image_surface_get_width(surface);
    hitbox_list_clear(hitbox_list);
    width = win_width / 2 - width / 2;

    int hit_box_width = 0;
    for (int i = 0; i < CARD_MAX_CUT_POSITIONS; i++) {
        if (i == scene->chosen_slots[0] || i == scene->chosen_slots[1]) {
            hit_box_width += layout_options->fan_spacing;
        } else {
            draw_card_back(
                renderer,
                layout_options->card_back,
                width + i * layout_options->fan_spacing,
                layout_options->top_offset,
                layout_options->card_width,
                layout_options->card_height
            );
            if (i < CARD_MAX_CUT_POSITIONS - 1) {
                hitbox_list_add_hitbox(
                    hitbox_list,
                    width + i * layout_options->fan_spacing,
                    layout_options->top_offset,
                    hit_box_width + layout_options->fan_spacing,
                    layout_options->card_height,
                    i + 1
                );
            } else {
                hitbox_list_add_hitbox(
                    hitbox_list,
                    width + i * layout_options->fan_spacing,
                    layout_options->top_offset,
                    layout_options->card_width,
                    layout_options->card_height,
                    i + 1
                );
            }
            hit_box_width = 0;
        }
    }

    if (scene->player_card != CARD_NONE) {
        draw_card(
            renderer,
            layout_options->card_images,
            scene->player_card,
            width,
            layout_options->middle_offset,
            layout_options->card_width,
            layout_options->card_height
        );

        draw_card(
            renderer,
            layout_options->card_images,
            scene->cpu_card,
            width + layout_options->fan_spacing * 12,
            layout_options->middle_offset,
            layout_options->card_width,
            layout_options->card_height
        );

        if ((scene->player_card & 0xf) < (scene->cpu_card & 0xf)) {
            draw_dialog(
                renderer,
                "You deal first.",
                hitbox_list,
                win_width / 2,
                layout_options->middle_offset,
                layout_options->padding,
                0
            );
        } else {
            draw_dialog(
                renderer,
                "CPU deals first.",
                hitbox_list,
                win_width / 2,
                layout_options->middle_offset,
                layout_options->padding,
                0
            );
        }
    } else {
        draw_dialog(
            renderer,
            "Choose a card. Lowest card deals first.",
            NULL,
            win_width / 2,
            layout_options->middle_offset,
            layout_options->padding,
            0
        );
    }
}

void scene_choose_crib(cairo_t *renderer, struct ChooseCribScene *scene, struct HitboxList *hitbox_list, struct LayoutOptions *layout_options) {
    int width = layout_options->fan_spacing * 5 + layout_options->card_width;
    cairo_surface_t *surface = cairo_get_target(renderer);
    int win_width = cairo_image_surface_get_width(surface);
    int middle = win_width / 2;
    width = middle - width / 2;

    for (int i = 0; i < 6; i++){
        draw_card_back(
            renderer,
            layout_options->card_back,
            width + i * layout_options->fan_spacing,
            layout_options->top_offset,
            layout_options->card_width,
            layout_options->card_height
        );
        draw_card(
            renderer,
            layout_options->card_images,
            scene->player_cards[i],
            width + i * layout_options->fan_spacing,
            layout_options->bottom_offset,
            layout_options->card_width,
            layout_options->card_height
        );
        hitbox_list_add_hitbox(
            hitbox_list,
            width + i * layout_options->fan_spacing,
            layout_options->bottom_offset,
            layout_options->card_width,
            layout_options->card_height,
            POSITION_NONE
        );
    }

    draw_card_back (
        renderer,
        layout_options->card_back,
        middle - (layout_options->card_width + layout_options->fan_spacing) * 2,
        layout_options->middle_offset,
        layout_options->card_width,
        layout_options->card_height
    );

    draw_dialog(
        renderer,
        "Choose two cards for the crib.",
        NULL,
        win_width / 2,
        layout_options->middle_offset,
        layout_options->padding,
        0
    );


}
