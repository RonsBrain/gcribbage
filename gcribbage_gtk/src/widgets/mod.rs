mod widgets;

use relm4::gtk::{
    self,
    glib::{self, Object},
};

glib::wrapper! {
    pub struct CardBox(ObjectSubclass<widgets::CardBox>)
        @extends gtk::DrawingArea, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl CardBox {
    pub fn new() -> Self {
        Object::builder().build()
    }
}

impl Default for CardBox {
    fn default() -> Self {
        Self::new()
    }
}
