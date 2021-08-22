use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

mod imp {
	use gtk::{Image, Label, ListBoxRow};

	use super::*;

	#[derive(Debug, Default, CompositeTemplate)]
	#[template(resource = "/dev/sp1rit/Utopia/ui/integration_item.ui")]
	pub struct UtopiaIntegrationItem {
		#[template_child]
		pub icon: TemplateChild<Image>,
		#[template_child]
		pub name: TemplateChild<Label>,
		#[template_child]
		pub uuid: TemplateChild<Label>
	}

	#[glib::object_subclass]
	impl ObjectSubclass for UtopiaIntegrationItem {
		type ParentType = ListBoxRow;
		type Type = super::UtopiaIntegrationItem;

		const NAME: &'static str = "UtopiaIntegrationItem";

		fn class_init(klass: &mut Self::Class) {
			Self::bind_template(klass);
		}

		fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
			obj.init_template();
		}
	}

	impl ObjectImpl for UtopiaIntegrationItem {
		fn constructed(&self, obj: &Self::Type) {
			self.parent_constructed(obj);
		}
	}

	impl WidgetImpl for UtopiaIntegrationItem {}
	impl ListBoxRowImpl for UtopiaIntegrationItem {}
}

glib::wrapper! {
	pub struct UtopiaIntegrationItem(ObjectSubclass<imp::UtopiaIntegrationItem>)
		@extends gtk::Widget, gtk::ListBoxRow,
		@implements gio::ActionMap, gio::ActionGroup;
}

impl UtopiaIntegrationItem {
	pub fn new() -> Self {
		glib::Object::new(&[]).expect("Failed to create UtopiaIntegrationItem")
	}

	pub fn init(&self, uuid: &str, name: &str, icon: Option<&str>) {
		let self_ = imp::UtopiaIntegrationItem::from_instance(self);
		self_.name.set_label(name);
		self_.uuid.set_label(uuid);
		self_.icon.set_icon_name(icon);
	}
}
