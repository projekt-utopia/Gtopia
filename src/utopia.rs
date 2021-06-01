use crate::{integration_item::UtopiaIntegrationItem, grid::UtopiaGrid};

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, gio, CompositeTemplate};

use gtk::glib::{clone, BindingFlags};

use libadwaita::{ApplicationWindow, NavigationDirection};

use std::{rc::Rc, cell::RefCell, collections::HashMap};

pub mod imp {
	use super::*;
	use libadwaita::{subclass::prelude::*, ApplicationWindow, Leaflet};

	use gtk::{ToggleButton, Button, Revealer, SearchEntry, Box, ListBox, Stack};
	#[derive(Debug, Default, CompositeTemplate)]
	#[template(resource = "/dev/sp1rit/Utopia/ui/window.ui")]
	pub struct UtopiaWindow {
		pub widgetmap: Rc<RefCell<HashMap<glib::GString, UtopiaGrid>>>,

		#[template_child]
		pub leaflet: TemplateChild<Leaflet>,
		#[template_child]
		pub leaflet_back: TemplateChild<Button>,

		#[template_child]
		pub sidebar_header: TemplateChild<gtk::HeaderBar>,

		#[template_child]
		pub search_btn: TemplateChild<ToggleButton>,
		#[template_child]
		pub search_revealer: TemplateChild<Revealer>,
		#[template_child]
		pub search: TemplateChild<SearchEntry>,

		#[template_child]
		pub sidebar: TemplateChild<Box>,
		#[template_child]
		pub module: TemplateChild<ListBox>,
		#[template_child]
		pub library: TemplateChild<Stack>,
	}

	#[glib::object_subclass]
	impl ObjectSubclass for UtopiaWindow {
		const NAME: &'static str = "UtopiaWindow";
		type Type = super::UtopiaWindow;
		type ParentType = ApplicationWindow;

		fn class_init(klass: &mut Self::Class) {
			Self::bind_template(klass);
		}

		fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
			obj.init_template();
		}
	}

	impl ObjectImpl for UtopiaWindow {
		fn constructed(&self, obj: &Self::Type) {
			let state = self.leaflet.is_folded();
			self.leaflet_back.set_visible(state);
			self.sidebar_header.set_show_title_buttons(state);

			obj.setup_search();
			obj.setup_stack();
			obj.populate();

			// TODO: this does not seem to work, as row select is called during init
			// self.leaflet.set_visible_child(&self.sidebar.get());

			self.parent_constructed(obj);
		}
	}

	impl WidgetImpl for UtopiaWindow {}
	impl WindowImpl for UtopiaWindow {}
	impl AdwWindowImpl for UtopiaWindow {}
	impl ApplicationWindowImpl for UtopiaWindow {}
	impl AdwApplicationWindowImpl for UtopiaWindow {}
}

glib::wrapper! {
	pub struct UtopiaWindow(ObjectSubclass<imp::UtopiaWindow>)
		@extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, ApplicationWindow,
		@implements gio::ActionMap, gio::ActionGroup;
}

impl UtopiaWindow {
	pub fn new<P: glib::IsA<gtk::Application>>(app: &P) -> Self {
		glib::Object::new(&[("application", app)]).expect("Failed to create UtopiaWindow")
	}

	pub fn setup_search(&self) {
		let self_ = imp::UtopiaWindow::from_instance(self);
		self_
			.search_btn.bind_property("active", &self_.search_revealer.get(), "reveal-child")
			.flags(BindingFlags::SYNC_CREATE | BindingFlags::BIDIRECTIONAL)
			.build();

		let search: gtk::SearchEntry = self_.search.get();
		self_
			.search_btn.connect_toggled(clone!(@weak search => move |btn| {
				if btn.is_active() {
					search.grab_focus();
				}
			}));
	}

	pub fn setup_stack(&self) {
		let self_ = imp::UtopiaWindow::from_instance(self);
		let stack: gtk::Stack = self_.library.get();
		let leaflet: libadwaita::Leaflet = self_.leaflet.get();
		let map = self_.widgetmap.clone();
		self_.module.connect_row_selected(clone!(@weak stack, @weak leaflet => move |_, item| {
			match item {
				Some(item) => {
					let map = map.borrow();
					let page = map.get(&item.widget_name()).unwrap();
					stack.set_visible_child(page);
					leaflet.navigate(NavigationDirection::Forward);
				},
				None => println!("Selection cleared")
			}
		}));
		let sidebar_header = self_.sidebar_header.get();
		let leaflet_back = self_.leaflet_back.get();
		self_.leaflet.connect_folded_notify(clone!(@weak leaflet, @weak leaflet_back, @weak sidebar_header =>
			move |_| {
				let state = leaflet.is_folded();
				leaflet_back.set_visible(state);
				sidebar_header.set_show_title_buttons(state);
			}
		));

		let leafleat = self_.leaflet.get();
		self_.leaflet_back.connect_clicked(clone!(@weak leafleat =>
			move |_| {
				leafleat.navigate(NavigationDirection::Back);
			}
		));
	}

	pub fn populate(&self) {
		let self_ = imp::UtopiaWindow::from_instance(self);
		let data = return_data();
		for provider in data {
			let item = UtopiaIntegrationItem::new();
				item.init(&provider.uuid, &provider.name, &provider.icon);
				item.set_widget_name(&provider.name);
			let page = UtopiaGrid::new();
				page.oldinit(provider.library);

			self_.module.append(&item);
			self_.library.add_child(&page);

			let mut map = self_.widgetmap.borrow_mut();
			map.insert(item.widget_name(), page);
		}
	}

	pub fn new_item(&self, item: utopia_common::library::LibraryItemFrontendDetails, mut sender: futures::channel::mpsc::Sender<crate::uev::UtopiaRequest>) {
		let self_ = imp::UtopiaWindow::from_instance(self);
		let mut map = self_.widgetmap.borrow_mut();
		for (uuid, (name, status)) in &item.providers {
			let card = crate::grid::card::UtopiaCard::new();
				card.meta(item.clone(), status);

			let key: glib::GString = uuid.to_owned().into();
			match map.get(&key) {
				Some(grid) => {
					grid.insert_card(&card);
				},
				None => {
					let item = UtopiaIntegrationItem::new();
						item.init(&key, &name, "");
						item.set_widget_name(&key);
					let page = UtopiaGrid::new();
						page.init(sender.clone());

					page.insert_card(&card);

					self_.module.append(&item);
					self_.library.add_child(&page);
					map.insert(item.widget_name(), page);
				}
			}
		}
		/*let key: glib::GString = item.uuid.into();
		match map.contains_key(&key) {
			true => {},
			false => {
				for (uuid, provider) in item.providers {

				}
			}
		}*/
		//println!("New item: {:?}", item);
	}
}

// ---

pub enum ItemStatus {
	Running,
	Installed,
	Downloading,
	Updating,
	Default
}

pub struct LibraryItem {
	pub title: String,
	pub status: ItemStatus,
	pub cover: String
}

struct Provider {
	pub name: String,
	pub uuid: String,
	pub icon: String,
	pub library: Vec<LibraryItem>
}

fn return_data() -> Vec<Provider> {
	vec![
		Provider {
			name: String::from("Steam"),
			uuid: String::from("com.valvesoftware.Steam"),
			icon: String::from("steam"),
			library: vec![
				LibraryItem {
					title: String::from("Half-Life"),
					status: ItemStatus::Installed,
					cover: String::from("hl1.png")
				},
				LibraryItem {
					title: String::from("Half-Life 2"),
					status: ItemStatus::Installed,
					cover: String::from("hl2.png")
				},
				LibraryItem {
					title: String::from("Counter-Strike: Source"),
					status: ItemStatus::Running,
					cover: String::from("css.png")
				},
				LibraryItem {
					title: String::from("Counter-Strike: Global Offensive"),
					status: ItemStatus::Updating,
					cover: String::from("csgo.png")
				},
				LibraryItem {
					title: String::from("NieR: Replicant"),
					status: ItemStatus::Default,
					cover: String::from("nier:replicant.png")
				},
				LibraryItem {
					title: String::from("NieR: Automata"),
					status: ItemStatus::Installed,
					cover: String::from("nier:automata.png")
				},
				LibraryItem {
					title: String::from("Doom Ethernal"),
					status: ItemStatus::Installed,
					cover: String::from("doom.png")
				}
			]
		},
		Provider {
			name: String::from("GOG"),
			uuid: String::from("pl.cdprojektred.GOG"),
			icon: String::from("wine"),
			library: vec![
				LibraryItem {
					title: String::from("The Witcher 1"),
					status: ItemStatus::Downloading,
					cover: String::from("tw1.png")
				},
				LibraryItem {
					title: String::from("The Witcher 2"),
					status: ItemStatus::Installed,
					cover: String::from("tw2.png")
				},
				LibraryItem {
					title: String::from("The Witcher 3"),
					status: ItemStatus::Installed,
					cover: String::from("tw3.png")
				}
			]
		}
	]
}
