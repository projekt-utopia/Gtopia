pub mod config;
pub mod utopia;
pub mod integration_item;
pub mod grid;
mod uev;
mod application;

use gtk::gio::prelude::*;
use gtk::{CssProvider, StyleContext, gdk::Display, STYLE_PROVIDER_PRIORITY_APPLICATION, glib, gio::{Resource, resources_register}};

fn main() {
	gtk::init().expect("Failed to initialize GTK");
	libadwaita::init();

	let res = Resource::load(config::PKGDATADIR.to_owned() + "/gtopia.gresource")
		.expect("Failed loading resources");
	resources_register(&res);

	glib::set_application_name("µtopia");
	glib::set_program_name(Some("µtopia"));

	let provider = CssProvider::new();
	provider.load_from_resource("/dev/sp1rit/Utopia/utopia.css");
	StyleContext::add_provider_for_display(&Display::default().expect("Error initializing gtk css provider."), &provider, STYLE_PROVIDER_PRIORITY_APPLICATION);

	let app = application::UtopiaFrontend::new();
	std::process::exit(app.run());
}
