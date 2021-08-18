pub mod card;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, gio, CompositeTemplate};

use libadwaita::subclass::prelude::*;

mod imp {
	use super::*;

	use libadwaita::Bin;
	use gtk::FlowBox;

	#[derive(Debug, Default, CompositeTemplate)]
	#[template(resource = "/dev/sp1rit/Utopia/ui/grid.ui")]
	pub struct UtopiaGrid {
		pub sender: once_cell::unsync::OnceCell<futures::channel::mpsc::Sender<crate::uev::UtopiaRequest>>,
		//pub dsender: once_cell::unsync::OnceCell<glib::Sender<Option<utopia_common::library::LibraryItemFrontendDetails>>>,

		pub items: std::cell::RefCell<std::collections::HashMap<String, card::UtopiaCard>>,

		#[template_child]
		pub grid: TemplateChild<FlowBox>,
	}

	#[glib::object_subclass]
	impl ObjectSubclass for UtopiaGrid {
		const NAME: &'static str = "UtopiaGrid";
		type Type = super::UtopiaGrid;
		type ParentType = Bin;

		fn class_init(klass: &mut Self::Class) {
			Self::bind_template(klass);
		}

		fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
			obj.init_template();
		}
	}

	impl ObjectImpl for UtopiaGrid {
		fn constructed(&self, obj: &Self::Type) {
			self.parent_constructed(obj);
		}
	}

	impl WidgetImpl for UtopiaGrid {}
	impl BinImpl for UtopiaGrid {}
}

glib::wrapper! {
	pub struct UtopiaGrid(ObjectSubclass<imp::UtopiaGrid>)
		@extends gtk::Widget, libadwaita::Bin,
		@implements gio::ActionMap, gio::ActionGroup;
}

impl UtopiaGrid {
	pub fn new() -> Self {
		glib::Object::new(&[]).expect("Failed to create UtopiaGrid")
	}
	pub fn setup_trigger(&self, dsender: glib::Sender<Option<utopia_common::library::LibraryItemFrontendDetails>>) {
		let self_ = imp::UtopiaGrid::from_instance(self);
		let sender = std::sync::Arc::new(std::sync::RwLock::new(self_.sender.clone()));
		self_.grid.connect_child_activated(move |_, child| {
			if let Err(e) = sender.write().unwrap().get_mut().unwrap().try_send(crate::uev::UtopiaRequest::TriggerLaunch(child.widget_name().into())) {
				eprintln!("Error requesting {} to launch: {}", child.widget_name(), e);
			}
		});

		self_.grid.connect_selected_children_changed(move |grid| {
			let item = match grid.selected_children().get(0) {
				Some(child) => Some(child.downcast_ref::<card::UtopiaCard>().unwrap().utopia().clone()),
				None => None
			};
			dsender.send(item).unwrap()
		});

		/* probably the worst sort function known to mankind */
		self_.grid.set_sort_func(move |b, n| {
			let bname = b.downcast_ref::<card::UtopiaCard>().unwrap().name();
			let bchars = bname.chars();
			let nname = n.downcast_ref::<card::UtopiaCard>().unwrap().name();
			let mut nchars = nname.chars();
			for r#char in bchars {
				let bint = r#char.to_ascii_uppercase() as u32;
				let nint = match nchars.next() {
					Some(int) => int.to_ascii_uppercase() as u32,
					None => return 1
				};
				if bint < nint {
					return -1;
				}
				if bint > nint {
					return 1;
				}
				// continue if equal
			}
			-1
		});
	}

	pub fn init(&self, sender: futures::channel::mpsc::Sender<crate::uev::UtopiaRequest>, dsender: glib::Sender<Option<utopia_common::library::LibraryItemFrontendDetails>>) {
		let self_ = imp::UtopiaGrid::from_instance(self);
		self_.sender.set(sender).expect("Failed setting up UtopiaGrid");
		//self_.dsender.set(dsender).expect("Failed setting up UtopiaGrid");
		self.setup_trigger(dsender);
	}
	pub fn insert_card(&self, uuid: String, card: &card::UtopiaCard) {
		let self_ = imp::UtopiaGrid::from_instance(self);
		self_.items.borrow_mut().insert(uuid, card.clone());
		self_.grid.insert(card, -1);
	}

	pub fn update_card(&self, uuid: &String, item: utopia_common::library::LibraryItemFrontend) {
		let self_ = imp::UtopiaGrid::from_instance(self);
		if let Some(card) = self_.items.borrow().get(uuid) {
			card.update(item);
		}
	}

	pub fn update_filter(&self, module: std::cell::Ref<Option<glib::GString>>, search: glib::GString) {
		let self_ = imp::UtopiaGrid::from_instance(self);
		let module = match module.as_ref() {
			Some(module) => Some(module.to_owned()),
			None => None
		};
		self_.grid.set_filter_func(move |card| {
			if let Some(module) = module.as_ref() {
				if !card.downcast_ref::<card::UtopiaCard>().unwrap().provider(module) {
					return false;
				}
			}
			if search != "" {
				return card.downcast_ref::<card::UtopiaCard>().unwrap().name().to_uppercase().contains(&search.to_uppercase());
			}

			true
		})
	}
}
