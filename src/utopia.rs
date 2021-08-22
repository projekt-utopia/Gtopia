use std::{cell::RefCell, collections::HashMap, rc::Rc};

use gtk::{gio,
          glib,
          glib::{clone, BindingFlags},
          prelude::*,
          subclass::prelude::*,
          CompositeTemplate};
use libadwaita::{ApplicationWindow, NavigationDirection};

use crate::{grid::UtopiaGrid, integration_item::UtopiaIntegrationItem};

#[derive(Debug, PartialEq)]
pub enum LeafletFoci {
	Providers,
	Library,
	Details
}
impl Default for LeafletFoci {
	fn default() -> Self {
		Self::Providers
	}
}

pub mod imp {
	use libadwaita::{subclass::prelude::*, ApplicationWindow, Leaflet};
	use gtk::{Box, Button, ListBox, Revealer, SearchEntry, ToggleButton};

	use super::*;
	#[derive(Debug, Default, CompositeTemplate)]
	#[template(resource = "/dev/sp1rit/Utopia/ui/window.ui")]
	pub struct UtopiaWindow {
		pub active_integration: Rc<RefCell<Option<glib::GString>>>,
		pub integrations: RefCell<Vec<String>>,
		pub widgetmap: Rc<RefCell<HashMap<glib::GString, UtopiaGrid>>>,

		pub lfoci: Rc<RefCell<LeafletFoci>>,

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
		pub game_leaflet: TemplateChild<Leaflet>,
		#[template_child]
		pub library: TemplateChild<crate::grid::UtopiaGrid>,
		#[template_child]
		pub detail: TemplateChild<crate::detail::UtopiaDetail>
	}

	#[glib::object_subclass]
	impl ObjectSubclass for UtopiaWindow {
		type ParentType = ApplicationWindow;
		type Type = super::UtopiaWindow;

		const NAME: &'static str = "UtopiaWindow";

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
			obj.setup_library();
			obj.setup_sidebar();
			//obj.populate();

			// TODO: this does not seem to work, as row select is called during
			// init self.leaflet.set_visible_child(&self.sidebar.get());

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
			.search
			.connect_search_changed(clone!(@weak self as utopia => move |_| {
				utopia.update_filter();
			}));
		self_
			.search_btn
			.bind_property("active", &self_.search_revealer.get(), "reveal-child")
			.flags(BindingFlags::SYNC_CREATE | BindingFlags::BIDIRECTIONAL)
			.build();

		let search: gtk::SearchEntry = self_.search.get();
		self_.search_btn.connect_toggled(clone!(@weak search => move |btn| {
			if btn.is_active() {
				search.grab_focus();
			} else {
				search.set_text("");
			}
		}));
	}

	pub fn setup_sidebar(&self) {
		let self_ = imp::UtopiaWindow::from_instance(self);
		//let ibuf =
		// gtk::IconPaintable::for_file(&gtk::gio::File::for_path("/home/
		// admin/workspace/Projects/µtopia/data/utopia-symbolic.png"), 512,
		// 512); println!("Iconsymb: {:?}", ibuf.is_symbolic());
		let icon = gtk::ImageBuilder::new()
			.icon_size(gtk::IconSize::Large)
			//.resource("/dev/sp1rit/Utopia/utopia.svg")
			//.paintable(&ibuf)
			.icon_name("dev.sp1rit.Gtopia-symbolic")
			.margin_top(8)
			.margin_bottom(8)
			.margin_start(8)
			.margin_end(8)
			.build();
		let label = gtk::LabelBuilder::new()
			.label("µtopia")
			.css_classes(vec![String::from("ititle")])
			.xalign(0.0)
			.margin_start(8)
			.margin_end(8)
			.build();
		let r#box = gtk::BoxBuilder::new().orientation(gtk::Orientation::Horizontal).build();
		r#box.append(&icon);
		r#box.append(&label);
		let all = gtk::ListBoxRowBuilder::new()
			.child(&r#box)
			.name("dev.sp1rit.Utopia.restricted.µtopia_all")
			.build();

		self_.module.append(&all);
	}

	pub fn setup_library(&self) {
		let self_ = imp::UtopiaWindow::from_instance(self);
		let game_leaflet = self_.game_leaflet.get();
		let library: crate::grid::UtopiaGrid = self_.library.get();
		let detail = self_.detail.get();
		let leaflet: libadwaita::Leaflet = self_.leaflet.get();
		let search = self_.search.get();
		let search_btn = self_.search_btn.get();
		let active_integration = self_.active_integration.clone();
		let lfoci = self_.lfoci.clone();
		self_.module.connect_row_selected(clone!(@weak self as utopia, @weak library, @weak leaflet, @weak search, @weak search_btn, @weak detail => move |_, item| {
			match item {
				Some(item) => {
					let name = item.widget_name();
					if name == "dev.sp1rit.Utopia.restricted.µtopia_all" {
						active_integration.replace(None);
					} else {
						active_integration.replace(Some(item.widget_name()));
					}
					//let map = map.borrow();
					//let page = map.get(&item.widget_name()).unwrap();
					//library.set_visible_child(page);
					utopia.update_filter();
					search_btn.set_active(false);
					detail.set_visible(false);
					leaflet.navigate(NavigationDirection::Forward);
					lfoci.replace(LeafletFoci::Library);
				},
				None => println!("Selection cleared")
			}
		}));
		let sidebar_header = self_.sidebar_header.get();
		let leaflet_back = self_.leaflet_back.get();
		//let game_leaflet = self_.game_leaflet.get();
		let lfoci = self_.lfoci.clone();
		self_.leaflet.connect_folded_notify(
			clone!(@weak leaflet, @weak leaflet_back, @weak sidebar_header /*@weak game_leaflet*/ =>
				move |_| {
					let state = leaflet.is_folded();
					leaflet_back.set_visible(state);
					sidebar_header.set_show_title_buttons(state);
					// it might be better to use game_leaflet.is_folded() rather then state
					// but it looks wierd if you go back
					if !state && lfoci.take() == LeafletFoci::Details {
						lfoci.replace(LeafletFoci::Library);
					}
				}
			)
		);

		let leafleat = self_.leaflet.get();
		let lfoci = self_.lfoci.clone();
		self_
			.leaflet_back
			.connect_clicked(clone!(@weak leafleat, @weak game_leaflet, @weak detail =>
				move |_| {
					match lfoci.take() {
						LeafletFoci::Providers => {},
						LeafletFoci::Library => {
							leafleat.navigate(NavigationDirection::Back);
							lfoci.replace(LeafletFoci::Providers);
						},
						LeafletFoci::Details => {
							/*game_leaflet.set_visible_child(&library);
							lfoci.replace(LeafletFoci::Library);*/
							detail.set_visible(false);
						}
					}
				}
			));

		let lfoci = self_.lfoci.clone();
		self_
			.detail
			.connect_visible_notify(glib::clone!(@weak game_leaflet, @weak library => move |detail| {
				if detail.get_visible() {
					game_leaflet.navigate(NavigationDirection::Forward);
					//set_visible_child(detail);
					lfoci.replace(LeafletFoci::Details);
				} else {
					game_leaflet.navigate(NavigationDirection::Back);
					//set_visible_child(&library);
					lfoci.replace(LeafletFoci::Library);
				}
			}));
	}

	pub fn init_listener(&self, sender: futures::channel::mpsc::Sender<crate::uev::UtopiaRequest>) {
		let self_ = imp::UtopiaWindow::from_instance(self);
		let (dsender, dreceiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
		self_.library.init(sender.clone(), dsender);
		self_.detail.init(sender, dreceiver);
	}

	pub fn update_filter(&self) {
		let self_ = imp::UtopiaWindow::from_instance(self);
		let selected_module = self_.active_integration.borrow();
		let search = self_.search.text();
		self_.library.update_filter(selected_module, search);
	}

	pub fn new_item(&self, item: utopia_common::library::LibraryItemFrontendDetails) {
		let self_ = imp::UtopiaWindow::from_instance(self);
		let mut integrations = self_.integrations.borrow_mut();
		let card = crate::grid::card::UtopiaCard::new();
		card.init(item.clone());
		self_.library.insert_card(item.uuid, &card);
		for (uuid, iprov) in &item.providers {
			if !integrations.contains(uuid) {
				let item = UtopiaIntegrationItem::new();
				item.init(&uuid, &iprov.name, iprov.icon.as_deref());
				item.set_widget_name(&uuid);
				self_.module.append(&item);
				integrations.push(uuid.to_owned());
			}
		}
	}

	pub fn update_item(&self, item: utopia_common::library::LibraryItemFrontend) {
		let self_ = imp::UtopiaWindow::from_instance(self);
		self_.library.update_card(&item.uuid.clone(), item);
	}
}
