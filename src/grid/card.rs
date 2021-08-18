use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, gio, CompositeTemplate};

mod imp {
	use super::*;

	use gtk::{FlowBoxChild, Frame, Overlay, Picture, Label};

	#[derive(Debug, Default, CompositeTemplate)]
	#[template(resource = "/dev/sp1rit/Utopia/ui/card.ui")]
	pub struct UtopiaCard {
		pub utopia: std::cell::RefCell<Option<utopia_common::library::LibraryItemFrontendDetails>>,

		#[template_child]
		pub frame: TemplateChild<Frame>,
		#[template_child]
		pub overlay: TemplateChild<Overlay>,
		#[template_child]
		pub coverimg: TemplateChild<Picture>,

		#[template_child]
		pub title: TemplateChild<Label>,
		#[template_child]
		pub status: TemplateChild<Label>
	}

	#[glib::object_subclass]
	impl ObjectSubclass for UtopiaCard {
		const NAME: &'static str = "UtopiaCard";
		type Type = super::UtopiaCard;
		type ParentType = FlowBoxChild;

		fn class_init(klass: &mut Self::Class) {
			Self::bind_template(klass);
		}

		fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
			obj.init_template();
		}
	}

	impl ObjectImpl for UtopiaCard {
		fn constructed(&self, obj: &Self::Type) {
			self.parent_constructed(obj);
		}
	}

	impl WidgetImpl for UtopiaCard {}
	impl FlowBoxChildImpl for UtopiaCard {}
}

glib::wrapper! {
	pub struct UtopiaCard(ObjectSubclass<imp::UtopiaCard>)
		@extends gtk::Widget, gtk::FlowBoxChild,
		@implements gio::ActionMap, gio::ActionGroup;
}

impl UtopiaCard {
	pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create UtopiaCard")
    }
	pub fn init(&self, item: utopia_common::library::LibraryItemFrontendDetails) {
		self.set_widget_name(&item.uuid);
		let self_ = imp::UtopiaCard::from_instance(self);
		self_.utopia.replace(Some(item.clone()));
		self_.title.set_label(&item.name);
		for status in item.active_provider.stati {
			self_.status.set_label(match status {
				utopia_common::library::LibraryItemStatus::Running(_pid) => "Running",
				utopia_common::library::LibraryItemStatus::Closing => "Closing",
				utopia_common::library::LibraryItemStatus::Updatable => "Update available",
				utopia_common::library::LibraryItemStatus::Updating => "Updating",
				utopia_common::library::LibraryItemStatus::Installed => "Installed"
			});
		}
		let size = 300;
		if item.details.artworks.len() == 0 {
			self_.coverimg.set_pixbuf(Some(&gtk::gdk_pixbuf::Pixbuf::from_resource_at_scale("/dev/sp1rit/Utopia/artwork.svg", (2*size)/3, size, true).unwrap()));
		}
		for artwork in item.details.artworks {
			match artwork.r#type {
				utopia_common::library::artwork::ArtworkType::CaseCover => {
					let buf = match artwork.data {
						utopia_common::library::artwork::ArtworkData::Data(data, has_alpha, bits_per_sample, width, height, rowstride) => {
							gtk::gdk_pixbuf::Pixbuf::from_bytes(&gtk::glib::Bytes::from(&data), gtk::gdk_pixbuf::Colorspace::Rgb, has_alpha, bits_per_sample, width, height, rowstride)
						},
						utopia_common::library::artwork::ArtworkData::Uri(_uri) => {
							//unimplemented!();
							gtk::gdk_pixbuf::Pixbuf::from_resource("/dev/sp1rit/Utopia/artwork.svg").unwrap()
						},
						utopia_common::library::artwork::ArtworkData::Path(path) => {
							gtk::gdk_pixbuf::Pixbuf::from_file(path).unwrap_or(
								gtk::gdk_pixbuf::Pixbuf::from_resource("/dev/sp1rit/Utopia/artwork.svg").unwrap()
							)
						}
					};
					let buf = buf.scale_simple((2*size)/3, size, gtk::gdk_pixbuf::InterpType::Bilinear);
					self_.coverimg.set_pixbuf(buf.as_ref());
				},
				_ => {}
			}
		}
	}

	pub fn utopia(&self) -> std::cell::Ref<utopia_common::library::LibraryItemFrontendDetails> {
		let self_ = imp::UtopiaCard::from_instance(self);
		let a = self_.utopia.borrow();
		let b: std::cell::Ref<utopia_common::library::LibraryItemFrontendDetails> = std::cell::Ref::map(a, |inner| {
			let details = inner.as_ref().unwrap();
			details
		});
		b
		//a.as_ref().unwrap()
	}
	pub fn provider(&self, provider: &glib::GString) -> bool {
		for (iprovider, _) in &self.utopia().providers {
			if provider == iprovider {
				// TODO: set active provider to provider
				return true;
			}
		}
		false
	}
	pub fn name(&self) -> String {
		self.utopia().name.clone()
	}

	pub fn update(&self, item: utopia_common::library::LibraryItemFrontend) {
		let self_ = imp::UtopiaCard::from_instance(self);
		self_.title.set_label(&item.name);
		for status in item.active_provider.stati.clone() {
			self_.status.set_label(match status {
				utopia_common::library::LibraryItemStatus::Running(_pid) => "Running",
				utopia_common::library::LibraryItemStatus::Closing => "Closing",
				utopia_common::library::LibraryItemStatus::Updatable => "Update available",
				utopia_common::library::LibraryItemStatus::Updating => "Updating",
				utopia_common::library::LibraryItemStatus::Installed => "Installed"
			});
		}
		let mut utopia = self_.utopia.borrow_mut();
		if let Some(details) = utopia.as_mut() {
			details.name = item.name;
			details.kind = item.kind;
			details.active_provider = item.active_provider;
			details.providers = item.providers;
		}

	}
}
