use crate::config;
use crate::utopia::UtopiaWindow;
use crate::uev::handle_event;

use gtk::prelude::*;
use gtk::subclass::prelude::*;

use gtk::glib::{self, WeakRef};
use gtk::{gio, Application};

use once_cell::unsync::OnceCell;

use std::rc::Rc;

mod imp {
	use super::*;

	#[derive(Default, Debug)]
	pub struct UtopiaFrontend {
		pub window: OnceCell<WeakRef<UtopiaWindow>>,
		pub utopia: Rc<std::cell::RefCell<Option<crate::uev::UtopiaEvents>>>
	}

	#[glib::object_subclass]
	impl ObjectSubclass for UtopiaFrontend {
		const NAME: &'static str = "UtopiaFrontend";
		type Type = super::UtopiaFrontend;
		type ParentType = Application;
	}

	impl ObjectImpl for UtopiaFrontend {}
	impl ApplicationImpl for UtopiaFrontend {
		fn activate(&self, application: &Self::Type) {
			let window = application.get_main_window();
			window.show();
			window.present();
		}

		fn startup(&self, application: &Self::Type) {
			self.parent_startup(application);
			application.set_resource_base_path(Some("/dev/sp1rit/Utopia/"));

			let (uev, tx, rx) = crate::uev::UtopiaEvents::new();
            self.utopia.replace(Some(uev));

			let window = UtopiaWindow::new(application);
			window.set_title(Some("Utopia"));
			self.window.set(window.downgrade()).expect("Failed to init application window");

			let txw = tx.clone();
			rx.attach(None, move |msg| handle_event(msg, tx.clone(), window.downgrade().clone().upgrade().unwrap()));
			application.get_main_window().init_listener(txw);
            self.utopia.borrow().as_ref().unwrap().start();
            self.utopia.borrow_mut().as_mut().unwrap().request_library();
            //self.utopia.borrow_mut().as_mut().unwrap().request_library();

			application.setup_actions();
            application.setup_accels();
		}
	}

	impl GtkApplicationImpl for UtopiaFrontend {}
}

glib::wrapper! {
	pub struct UtopiaFrontend(ObjectSubclass<imp::UtopiaFrontend>)
		@extends gio::Application, Application,
		@implements gio::ActionGroup, gio::ActionMap;
}

impl UtopiaFrontend {
	pub fn new() -> Self {
		glib::Object::new(&[
			("application-id", &config::APP_ID.to_owned()),
			("flags", &gio::ApplicationFlags::empty()),
		]).unwrap()

	}

	fn get_main_window(&self) -> UtopiaWindow {
		let imp = imp::UtopiaFrontend::from_instance(self);
		imp.window.get().unwrap().clone().upgrade().unwrap()
	}

	fn setup_actions(&self) {
		let quit = gio::SimpleAction::new("quit", None);
		quit.connect_activate(glib::clone!(@weak self as app => move |_, _| {
			app.quit()
		}));
		self.add_action(&quit);

		let about = gio::SimpleAction::new("about", None);
		about.connect_activate(glib::clone!(@weak self as app => move |_, _| {
			app.show_about_diag()
		}));
		self.add_action(&about);
	}

	fn setup_accels(&self) {
		self.set_accels_for_action("app.quit", &["<Primary>q"]);
	}

	fn show_about_diag(&self) {
		let win = self.get_main_window();
		let authors = vec![String::from("Florian \"sp1rit\" <sp1rit@disroot.org>")];

		let diag = gtk::AboutDialogBuilder::new()
			.authors(authors)
			.icon_name(config::APP_ID)
			.comments("GTK frontend to µtopia")
			.license_type(gtk::License::Agpl30)
			.wrap_license(true)
			.version("0.0.0-devel")
			.website("https://github.com/projekt-utopia/")
			.website_label("GH: @projekt-utopia")
			.copyright(&format!("Copyright (c) 2021 Florian \"sp1rit\" and contributors"))
			.build();

		diag.set_transient_for(Some(&win));
		diag.set_modal(true);
		diag.show();
	}
}
