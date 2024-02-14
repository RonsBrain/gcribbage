use gcribbage_lib::deck::{Card, Rank, Suit};
use relm4::gtk::{
    self,
    cairo::Context,
    gdk::prelude::GdkCairoContextExt,
    gdk_pixbuf::Pixbuf,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};
use std::cell::RefCell;
use std::collections::HashMap;

/// A structure for holding a Pixbuf that represents a deck of cards.
struct CardBuffer {
    buffer: Option<Pixbuf>,
    sub_pixbufs: HashMap<Card, Pixbuf>,
}

impl CardBuffer {
    pub fn has_buffer(&self) -> bool {
        self.buffer.is_some()
    }

    /// Create a Pixbuf of the given height that has the card images as
    /// loaded from an SVG.
    ///
    /// The width will match the aspect ration with respect to the height.
    pub fn create_buffer(&mut self, height: i32) {
        // We're going to rebuild the card images, so clear
        // what we already have.
        self.sub_pixbufs.clear();
        self.buffer = Some(
            Pixbuf::from_resource_at_scale(
                "/com/ronsbrain/gcribbage_gtk/anglo.svg",
                -1, // Make the width match the origin aspect ratio wrt the height
                height * 5,
                true,
            )
            .expect("Could not create card buffer"),
        );

        // Build out the HashMap of Card to sub pixbufs
        // that represent the card images.
        let buffer = self.buffer.as_ref().unwrap();
        let width = buffer.width() / 13;
        let height = buffer.height() / 5;
        for suit in Suit::iter() {
            // The iterator for Suit doesn't go in the order
            // that we expect for the SVG, so I guess we just
            // remap it for now.
            //
            // TODO: Eliminate this logic and either make the
            // iterator match the suit sequence in the SVG or
            // figure out if there's a way to use the SVG id's
            // to pick out individual cards.
            let row = match suit {
                Suit::Clubs => 0,
                Suit::Diamonds => 1,
                Suit::Hearts => 2,
                Suit::Spades => 3,
            };
            for rank in Rank::iter() {
                let col = rank.ordinal() - 1;
                let sub_pixbuf =
                    buffer.new_subpixbuf(width * col as i32, height * row, width, height);
                self.sub_pixbufs.insert(Card::new(suit, rank), sub_pixbuf);
            }
        }
    }

    pub fn get_pixbuf_for(&self, card: Card) -> &Pixbuf {
        self.sub_pixbufs
            .get(&card)
            .expect("Requested pixbuf for imaginary card")
    }
}

impl Default for CardBuffer {
    fn default() -> Self {
        Self {
            buffer: None,
            sub_pixbufs: HashMap::new(),
        }
    }
}

#[derive(Default)]
pub struct CardBox {
    // Each CardBox will have its own CardBuffer. This can cause
    // some duplication but
    //  - It's better than having a static reference and unsafe calls
    //  - It's possible that there are use cases that want to have
    //    differently sized cards
    card_buffer: RefCell<CardBuffer>,
}

#[glib::object_subclass]
impl ObjectSubclass for CardBox {
    const NAME: &'static str = "CardBox";
    type Type = super::CardBox;
    type ParentType = gtk::DrawingArea;
}

impl CardBox {
    fn draw(&self, cr: &Context, _width: i32, height: i32) {
        let mut buffer = self.card_buffer.borrow_mut();
        if !buffer.has_buffer() {
            buffer.create_buffer(height);
        }
        for (i, rank) in Rank::iter().enumerate() {
            let pixbuf = buffer.get_pixbuf_for(Card::new(Suit::Spades, rank));
            GdkCairoContextExt::set_source_pixbuf(cr, pixbuf, i as f64 * 20.0, 0.0);
            cr.paint().expect("Could not paint");
        }
    }
}

impl ObjectImpl for CardBox {
    fn constructed(&self) {
        self.parent_constructed();
        // We need to set up the draw function here. We're going to defer
        // to the widget's draw function.
        DrawingAreaExtManual::set_draw_func(
            self.obj().as_ref(),
            clone!(@weak self as widget => move |_, cr, w, h| widget.draw(cr, w, h)),
        );
    }
}
impl WidgetImpl for CardBox {}
impl DrawingAreaImpl for CardBox {
    fn resize(&self, _width: i32, height: i32) {
        // When resizing the widget, reload the cards and recalculate the
        // card sub-pixmaps. We want the cards to be proportional with
        // respect to the widget size.
        let mut buffer = self.card_buffer.borrow_mut();
        buffer.create_buffer(height);
    }
}
