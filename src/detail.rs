use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use crate::uev::UtopiaRequest;

mod imp {
	use gtk::{Box, Button, ComboBox, Label, Picture};

	use super::*;

	#[derive(Debug, Default, CompositeTemplate)]
	#[template(resource = "/dev/sp1rit/Utopia/ui/detail.ui")]
	pub struct UtopiaDetail {
		pub running: std::rc::Rc<std::cell::Cell<bool>>,
		pub current_uuid: std::rc::Rc<std::cell::RefCell<Option<String>>>,
		pub current_module: std::rc::Rc<std::cell::RefCell<Option<String>>>,
		pub sender: once_cell::unsync::OnceCell<futures::channel::mpsc::Sender<crate::uev::UtopiaRequest>>,

		pub actions: gio::SimpleActionGroup,
		pub kill_action: once_cell::unsync::OnceCell<gio::SimpleAction>,

		#[template_child]
		pub cover: TemplateChild<Picture>,
		#[template_child]
		pub name: TemplateChild<Label>,
		#[template_child]
		pub uuid: TemplateChild<Label>,
		#[template_child]
		pub dinfos: TemplateChild<ComboBox>,

		#[template_child]
		pub hide_btn: TemplateChild<Button>,
		#[template_child]
		pub primary_btn: TemplateChild<Button>
	}

	#[glib::object_subclass]
	impl ObjectSubclass for UtopiaDetail {
		type ParentType = Box;
		type Type = super::UtopiaDetail;

		const NAME: &'static str = "UtopiaDetail";

		fn class_init(klass: &mut Self::Class) {
			Self::bind_template(klass);
		}

		fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
			obj.init_template();
		}
	}

	impl ObjectImpl for UtopiaDetail {
		fn constructed(&self, obj: &Self::Type) {
			obj.setup_actions();
			self.parent_constructed(obj);
			obj.setup_triggers();
		}
	}

	impl WidgetImpl for UtopiaDetail {}
	impl BoxImpl for UtopiaDetail {}
}

glib::wrapper! {
	pub struct UtopiaDetail(ObjectSubclass<imp::UtopiaDetail>)
		@extends gtk::Widget, gtk::Box,
		@implements gio::ActionMap, gio::ActionGroup;
}

impl UtopiaDetail {
	pub fn new() -> Self {
		glib::Object::new(&[]).expect("Failed to create UtopiaDetail")
	}

	pub fn setup_triggers(&self) {
		let self_ = imp::UtopiaDetail::from_instance(self);
		self_
			.hide_btn
			.connect_clicked(glib::clone!(@weak self as detail => move |_| {
				detail.set_visible(false);
			}));

		self_
			.primary_btn
			.connect_clicked(glib::clone!(@strong self as detail => move |_| {
				let self_ = imp::UtopiaDetail::from_instance(&detail);
				if let Some(uuid) = self_.current_uuid.borrow().as_ref() {
					let request = match self_.running.get() {
						false => UtopiaRequest::TriggerLaunch(uuid.into()),
						true => UtopiaRequest::TriggerClose(utopia_common::library::LibraryItemProviderQuitActions::ActiveProvider(uuid.into()))
					};
					if let Err(e) = self_.sender.clone().get_mut().unwrap().try_send(request) {
						eprintln!("Error requesting to close {}: {}", uuid, e);
					}
				}
			}));

		self_
			.dinfos
			.connect_changed(glib::clone!(@strong self as detail => move |infos| {
				if let Some(active) = infos.active_id() {
					let self_ = imp::UtopiaDetail::from_instance(&detail);
					if let Some(uuid) = self_.current_uuid.borrow().as_ref() {
						if let Some(provider_uuid) = self_.current_module.borrow().as_ref() {
							if provider_uuid != &active {
								if let Err(e) = self_.sender.clone().get_mut().unwrap().try_send(
									UtopiaRequest::TriggerProviderUpdate(uuid.into(), active.clone().into())
								) {
									eprintln!("Error requesting {} to launch: {}", provider_uuid, e);
								}
							}
						}
					}
					self_.current_module.replace(Some(active.into()));
				}
			}));
	}

	fn setup_actions(&self) {
		let self_ = imp::UtopiaDetail::from_instance(self);
		self.insert_action_group("detail", Some(&self_.actions));

		let pref = gio::SimpleAction::new("preferences", None);

		pref.connect_activate(glib::clone!(@strong self as detail => move |_, _| {
			let self_ = imp::UtopiaDetail::from_instance(&detail);
			if let Some(uuid) = self_.current_uuid.borrow().as_ref() {
				if let Some(current_module) = self_.current_module.borrow().as_ref() {
					if let Err(e) = self_.sender.clone().get_mut().unwrap().try_send(
						UtopiaRequest::TriggerPreferenceDiag(current_module.into(), uuid.into())
					) {
						eprintln!("Error trying to request preference diag from {} for {}: {}", uuid, current_module, e);
					}
				}
			}
		}));

		let kill_action = gio::SimpleAction::new("kill", None);

		kill_action.connect_activate(glib::clone!(@strong self as detail => move |_, _| {
			let self_ = imp::UtopiaDetail::from_instance(&detail);
			if let Some(uuid) = self_.current_uuid.borrow().as_ref() {
				if self_.running.get() {
					if let Err(e) = self_.sender.clone().get_mut().unwrap().try_send(
						UtopiaRequest::TriggerKill(utopia_common::library::LibraryItemProviderQuitActions::ActiveProvider(uuid.into()))
					) {
						eprintln!("Error requesting to close {}: {}", uuid, e);
					}
				};
			}
		}));

		&self_.actions.add_action(&pref);
		&self_.actions.add_action(&kill_action);

		&self_.kill_action.set(kill_action).unwrap();
	}

	pub fn init(
		&self,
		sender: futures::channel::mpsc::Sender<UtopiaRequest>,
		listener: glib::Receiver<crate::grid::SidebarMsg>
	) {
		let self_ = imp::UtopiaDetail::from_instance(self);
		self_.sender.set(sender).expect("Failed setting up UtopiaDetail");
		let cover = self_.cover.get();
		let name = self_.name.get();
		let uuid = self_.uuid.get();
		let info = self_.dinfos.get();
		let current_uuid = self_.current_uuid.clone();
		let current_module = self_.current_module.clone();
		let primary_btn = self_.primary_btn.get();
		let kill_action = self_.kill_action.get().unwrap();
		let running = self_.running.clone();
		listener.attach(None, glib::clone!(@weak self as detail, @weak cover, @weak name, @weak uuid, @weak info, @weak primary_btn, @weak kill_action => @default-return glib::Continue(false), move |msg| {
    		match msg.item {
    			Some(item) => {
    				if msg.action == crate::grid::SidebarMsgAction::Update && current_uuid.borrow().as_ref() != Some(item.uuid.clone()).as_ref() {
    					println!("TRACE: non-selected item got updated");
    					return glib::Continue(true)
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
								let size = 360;
								let buf = buf.scale_simple((2*size)/3, size, gtk::gdk_pixbuf::InterpType::Bilinear);
								cover.set_pixbuf(buf.as_ref());
							},
							_ => {}
						}
					}

					name.set_label(&item.name);
					uuid.set_label(&item.uuid);

					current_uuid.replace(Some(item.uuid));
    				current_module.replace(Some(item.active_provider.uuid.clone()));

					info.clear();
    				let prov = gtk::ListStore::new(&[glib::types::Type::STRING, glib::types::Type::STRING, glib::types::Type::STRING]);
    				for (iuuid, iprov) in item.providers {
    					prov.insert_with_values(None, &[(0, &iuuid), (1, &iprov.icon.unwrap_or(String::from(""))), (2, &iprov.name)]);
    				}
    				info.set_model(Some(&prov));
    				info.set_id_column(0);
    				let icon = gtk::CellRendererPixbufBuilder::new()
    					.icon_size(gtk::IconSize::Large)
    					.build();
    				info.pack_start(&icon, false);
    				info.add_attribute(&icon, "icon-name", 1);
    				let text = gtk::CellRendererTextBuilder::new()
    					.ellipsize(gtk::pango::EllipsizeMode::End)
    					.ellipsize_set(true)
    					.build();
    				info.pack_start(&text, false);
    				info.add_attribute(&text, "text", 2);

    				info.set_active_id(Some(&item.active_provider.uuid));
    				detail.set_visible(true);

    				if item.active_provider.stati.iter().any(|&i| std::mem::discriminant(&i) == std::mem::discriminant(&utopia_common::library::LibraryItemStatus::Running(None))) {
						running.set(true);
						primary_btn.set_label("Stop");

					} else {
						running.set(false);
						primary_btn.set_label("Launch");
					}
					if item.active_provider.stati.iter().any(|&i| {
						if let utopia_common::library::LibraryItemStatus::Running(Some(_)) = i {
							true
						} else {
							false
						}
					}) {
						kill_action.set_enabled(true);
					} else {
						kill_action.set_enabled(false);
					}
    			},
    			None => detail.set_visible(false)
    		};
    		glib::Continue(true)
    	}));
	}
}
