use gtk::gio;
use gtk::prelude::*;
use relm4::gtk::subclass::drawing_area::DrawingAreaImpl;
use relm4::prelude::*;
mod widgets;
use gcribbage_lib::deck::Card;
use widgets::CardBox;

struct App {
    hand: Vec<Card>,
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = Vec<Card>;
    type Input = ();
    type Output = ();

    view! {
        gtk::Window {
            set_title: Some("Simple App"),
            set_default_size: (800, 600),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,
                set_homogeneous: true,

                CardBox { },
                CardBox { },
            }
        }
    }

    fn init(
        hand: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = App { hand };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {}
}

fn main() {
    gio::resources_register_include!("resources.gresource").expect("Failed to register resources");
    let app = RelmApp::new("relm4.example.simple");
    app.run::<App>(vec![]);
}
