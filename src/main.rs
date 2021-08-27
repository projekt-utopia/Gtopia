mod application;
pub mod config;
mod detail;
pub mod grid;
pub mod integration_item;
mod uev;
pub mod utopia;

pub mod preferences;

use gtk::{gdk::Display,
          gio::{prelude::*, resources_register, Resource},
          glib,
          CssProvider,
          StyleContext,
          STYLE_PROVIDER_PRIORITY_APPLICATION};

fn main() {
	gtk::init().expect("Failed to initialize GTK");
	libadwaita::init();

	let res = Resource::load(config::PKGDATADIR.to_owned() + "/gtopia.gresource").expect("Failed loading resources");
	resources_register(&res);

	glib::set_application_name("µtopia");
	glib::set_program_name(Some(&config::APP_ID));
	gtk::Window::set_default_icon_name(config::APP_ID);

	let provider = CssProvider::new();
	provider.load_from_resource("/dev/sp1rit/Utopia/utopia.css");
	StyleContext::add_provider_for_display(
		&Display::default().expect("Error initializing gtk css provider."),
		&provider,
		STYLE_PROVIDER_PRIORITY_APPLICATION
	);

	let app = application::UtopiaFrontend::new();
	std::process::exit(app.run());
}
