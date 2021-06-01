pub mod card;
use crate::utopia::LibraryItem;

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
    pub fn setup_trigger(&self) {
    	let self_ = imp::UtopiaGrid::from_instance(self);
    	let sender = std::sync::Arc::new(std::sync::RwLock::new(self_.sender.clone()));
    	self_.grid.connect_child_activated(move |_, child| {
    		if let Err(e) = sender.write().unwrap().get_mut().unwrap().try_send(crate::uev::UtopiaRequest::TriggerLaunch(child.widget_name().into())) {
    			eprintln!("Error requesting {} to launch: {}", child.widget_name(), e);
    		}
    	});
    }

	pub fn oldinit(&self, items: Vec<LibraryItem>) {
		let self_ = imp::UtopiaGrid::from_instance(self);
		for item in items {
			let card = card::UtopiaCard::new();
			self_.grid.insert(&card, -1);
			card.init(item);
		}
	}
	pub fn init(&self, sender: futures::channel::mpsc::Sender<crate::uev::UtopiaRequest>) {
		let self_ = imp::UtopiaGrid::from_instance(self);
		self_.sender.set(sender).expect("Failed setting up UtopiaGrid");
		self.setup_trigger();
	}
	pub fn insert_card(&self, card: &card::UtopiaCard) {
		let self_ = imp::UtopiaGrid::from_instance(self);
		self_.grid.insert(card, -1);
	}
}
