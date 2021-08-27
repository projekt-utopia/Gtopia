use std::{ops::Deref,
          sync::{Arc, RwLock}};

use libadwaita::{prelude::*,
                 ActionRow,
                 ExpanderRow,
                 PreferencesGroup,
                 PreferencesGroupBuilder,
                 PreferencesPage,
                 PreferencesPageBuilder,
                 PreferencesWindow,
                 PreferencesWindowBuilder};
use gtk::{glib::{self, clone, Type},
          prelude::*,
          Align,
          Orientation};
use utopia_common::library::preferences::{self, FieldType, InputType};

pub type ValueStore = Arc<RwLock<std::collections::HashMap<String, preferences::FieldType>>>;

pub struct GtopiaPreferenceBuilder {
	diag: preferences::PreferenceDiag,
	values: ValueStore
}
impl GtopiaPreferenceBuilder {
	pub fn new(diag: preferences::PreferenceDiag, values: ValueStore) -> Self {
		GtopiaPreferenceBuilder {
			diag,
			values
		}
	}

	fn build_entry(&self, value: &String, uuid: String, purpose: gtk::InputPurpose) -> gtk::Entry {
		let buf = gtk::EntryBuffer::new(Some(&value));
		let entry = gtk::Entry::builder().buffer(&buf).input_purpose(purpose).build();

		let values = self.values.clone();
		buf.connect_text_notify(move |buf| {
			values
				.write()
				.unwrap()
				.insert(uuid.clone(), FieldType::Input(InputType::Text(buf.text())));
		});

		entry
	}

	fn build_table<F: Fn(&gtk::ListStore) -> FieldType + 'static>(
		&self,
		uuid: String,
		model: &gtk::ListStore,
		return_builder: &'static F
	) -> gtk::Box {
		let view = gtk::TreeView::with_model(model);
		view.set_headers_visible(false);
		view.set_grid_lines(gtk::TreeViewGridLines::Vertical);
		view.set_reorderable(true);
		view.set_hexpand(true);

		view.set_column_drag_function(Some(Box::new(|_, _, _, _| true)));

		for i in 0..model.n_columns() {
			let col = gtk::TreeViewColumn::new();
			let text = gtk::CellRendererText::new();
			text.set_editable(true);
			col.pack_start(&text, false);
			col.add_attribute(&text, "text", i as i32);
			view.append_column(&col);

			text.connect_edited(clone!(@weak model => move |_, path, changed| {
				let iter = model.iter(&path).unwrap();
				let changed: glib::GString = changed.into();
				model.set(&iter, &[(i as u32, &changed)])
			}));
		}

		let update_settings_store = move |model: &gtk::ListStore, clone: &(ValueStore, String)| {
			let (values, uuid) = clone;
			let field = return_builder(model);

			{
				values.write().unwrap().insert(uuid.to_owned(), field);
			}
		};

		let changed = (self.values.clone(), uuid.clone());
		let deleted = (self.values.clone(), uuid);
		model.connect_row_changed(move |model, _, _| update_settings_store(model, &changed));
		model.connect_row_deleted(move |model, _| update_settings_store(model, &deleted));

		let add_btn = gtk::Button::from_icon_name(Some("list-add-symbolic"));
		let rm_btn = gtk::Button::from_icon_name(Some("list-remove-symbolic"));
		add_btn.set_css_classes(&["toolbtn_like_btn", "image-button"]);
		rm_btn.set_css_classes(&["toolbtn_like_btn", "image-button"]);

		add_btn.connect_clicked(clone!(@weak model => move |_| {
			let row = model.append();
			let empty_str: glib::GString = "".into();
			let types: Vec<(u32, &dyn glib::ToValue)> = (0..model.n_columns())
				.map(|i| (i as u32, &empty_str as _))
				.collect();

			model.set(&row, types.deref());
		}));
		rm_btn.connect_clicked(clone!(@weak view, @weak model => move |_| {
			let selection = view.selection();
			if let Some(selected) = selection.selected() {
				model.remove(&selected.1);
			}
		}));

		let toolbar = gtk::Box::new(Orientation::Horizontal, 4);
		toolbar.set_halign(Align::End);
		toolbar.set_margin_top(3);
		toolbar.append(&add_btn);
		toolbar.append(&rm_btn);

		let scrollable = gtk::ScrolledWindow::builder()
			.has_frame(false)
			.hscrollbar_policy(gtk::PolicyType::Automatic)
			.vscrollbar_policy(gtk::PolicyType::Never)
			.child(&view)
			.build();

		let container = gtk::Box::new(Orientation::Vertical, 0);
		container.append(&scrollable);
		container.append(&toolbar);

		container
	}

	fn build_action_row<P: IsA<gtk::Widget>>(item: &preferences::InputField, widget: &P) -> libadwaita::ActionRow {
		widget.set_halign(Align::Center);
		widget.set_valign(Align::Center);

		let row = ActionRow::new();
		row.set_title(Some(&item.title));
		row.set_subtitle(item.subtitle.as_deref());
		row.add_suffix(widget);
		row.set_activatable_widget(Some(widget));

		row
	}

	fn build_expander_row<P: IsA<gtk::Widget>>(item: &preferences::InputField, widget: &P) -> libadwaita::ExpanderRow {
		widget.set_halign(Align::Fill);
		widget.set_valign(Align::Start);

		let row = ExpanderRow::new();
		row.set_title(Some(&item.title));
		row.set_subtitle(item.subtitle.as_deref());
		row.add(widget);
		row.set_enable_expansion(true);

		row
	}

	fn build_item(&self, item: &preferences::InputField) -> libadwaita::PreferencesRow {
		let values = self.values.clone();
		let uuid = item.uuid.clone();

		match &item.r#type {
			FieldType::Input(input) => match input {
				InputType::Text(value) => {
					Self::build_action_row(item, &self.build_entry(value, uuid, gtk::InputPurpose::FreeForm)).upcast()
				},
				InputType::Email(value) => {
					Self::build_action_row(item, &self.build_entry(value, uuid, gtk::InputPurpose::Email)).upcast()
				},
				InputType::Phone(value) => {
					Self::build_action_row(item, &self.build_entry(value, uuid, gtk::InputPurpose::Phone)).upcast()
				},
				InputType::Url(value) => {
					Self::build_action_row(item, &self.build_entry(value, uuid, gtk::InputPurpose::Url)).upcast()
				},
				InputType::Password(value) => {
					let entry = gtk::PasswordEntry::new();
					entry.set_show_peek_icon(true);
					entry.set_text(&value);

					entry.connect_text_notify(move |buf| {
						values
							.write()
							.unwrap()
							.insert(uuid.clone(), FieldType::Input(InputType::Text(buf.text().to_string())));
					});

					Self::build_action_row(item, &entry).upcast()
				},
				InputType::Number(num) => {
					let spin = gtk::SpinButton::with_range(num.range.0, num.range.1, num.step);
					spin.set_value(num.value);

					let num = *num;
					spin.connect_value_changed(move |spin| {
						let mut num = num;
						num.value = spin.value();
						{
							values
								.write()
								.unwrap()
								.insert(uuid.clone(), FieldType::Input(InputType::Number(num)));
						}
					});

					Self::build_action_row(item, &spin).upcast()
				}
			},
			FieldType::Checkbox(value) => {
				let switch = gtk::Switch::builder().active(*value).build();

				switch.connect_state_set(move |switch, value| {
					{
						values.write().unwrap().insert(uuid.clone(), FieldType::Checkbox(value));
					}
					switch.set_state(value);
					gtk::Inhibit {
						0: true
					}
				});

				Self::build_action_row(item, &switch).upcast()
			},
			FieldType::Dropdown(index, dvalues) => {
				let model = gtk::ListStore::new(&[gtk::glib::types::Type::STRING]);
				for value in dvalues {
					model.insert_with_values(None, &[(0, value)]);
				}

				let combo = gtk::ComboBox::with_model(&model);
				combo.set_id_column(0);
				let text = gtk::CellRendererText::new();
				combo.pack_start(&text, false);
				combo.add_attribute(&text, "text", 0);

				let dv: &Vec<String> = dvalues;
				combo.set_active_id(dv.get(*index as usize).map(|string| string.deref()));

				let dvalues = dvalues.clone();
				combo.connect_changed(move |combo| {
					if let Some(active) = combo.active_id() {
						{
							values.write().unwrap().insert(
								uuid.clone(),
								FieldType::Dropdown(
									dvalues.iter().position(|v| v == &active).unwrap_or(0),
									dvalues.clone()
								)
							);
						}
					}
				});

				Self::build_action_row(item, &combo).upcast()
			},
			FieldType::List(items) => {
				let model = gtk::ListStore::new(&[Type::STRING]);
				for item in items {
					model.insert_with_values(None, &[(0, item)]);
				}

				let view = self.build_table(uuid, &model, &|model| {
					let mut state = Vec::new();
					if let Some(iter) = model.iter_children(None) {
						state.push(model.get(&iter, 0).get().unwrap());
						while model.iter_next(&iter) {
							state.push(model.get(&iter, 0).get().unwrap());
						}
					}

					FieldType::List(state)
				});
				Self::build_expander_row(item, &view).upcast()
			},
			FieldType::KeyValueList(list) => {
				let model = gtk::ListStore::new(&[Type::STRING, Type::STRING]);
				for item in list {
					model.insert_with_values(None, &[(0, item.0), (1, item.1)]);
				}

				let view = self.build_table(uuid, &model, &|model| {
					let mut map = std::collections::HashMap::new();
					if let Some(iter) = model.iter_children(None) {
						map.insert(model.get(&iter, 0).get().unwrap(), model.get(&iter, 1).get().unwrap());
						while model.iter_next(&iter) {
							map.insert(model.get(&iter, 0).get().unwrap(), model.get(&iter, 1).get().unwrap());
						}
					}

					FieldType::KeyValueList(map)
				});
				Self::build_expander_row(item, &view).upcast()
			}
		}
	}

	fn build_group(&self, group: &preferences::PreferenceGroup) -> PreferencesGroup {
		let grp = PreferencesGroupBuilder::new().title(&group.title).build();
		grp.set_description(group.description.as_deref());

		for item in &group.fields {
			grp.add(&self.build_item(item));
		}

		grp
	}

	fn build_page(&self, pane: &preferences::PreferencePane) -> PreferencesPage {
		let page = PreferencesPageBuilder::new().title(&pane.title).build();
		page.set_icon_name(pane.icon.as_deref());

		for group in &pane.groups {
			page.add(&self.build_group(group));
		}

		page
	}

	pub fn build(&self) -> (PreferencesWindow, gtk::Button) {
		let win = PreferencesWindowBuilder::new()
			.can_swipe_back(true)
			.search_enabled(true)
			.build();

		for page in &self.diag.panes {
			win.add(&self.build_page(page));
		}

		let save_btn = gtk::Button::with_label("Save");
		if let Some(leaflet) = libadwaita::prelude::WindowExt::child(&win)
			.map(|leaflet| leaflet.downcast::<libadwaita::Leaflet>().ok())
			.flatten()
		{
			if let Some(headerbar) = leaflet
				.visible_child()
				.map(|gtk_box| {
					gtk_box
						.first_child()
						.map(|headerbar| headerbar.downcast::<libadwaita::HeaderBar>().ok())
				})
				.flatten()
				.flatten()
			{
				if let Some(search_btn) = headerbar
					.first_child()
					.map(|gtk_window_handle| {
						gtk_window_handle.first_child().map(|gtk_center_box| {
							gtk_center_box.last_child().map(|adw_gizmo| {
								adw_gizmo.first_child().map(|gtk_box| {
									gtk_box
										.first_child()
										.map(|toggle_button| toggle_button.downcast::<gtk::ToggleButton>().ok())
								})
							})
						})
					})
					.flatten()
					.flatten()
					.flatten()
					.flatten()
					.flatten()
				{
					headerbar.remove(&search_btn);
					headerbar.pack_start(&search_btn);

					search_btn.connect_toggled(clone!(@weak save_btn => move |btn| {
						let active = btn.is_active();
						save_btn.set_visible(!active);
					}));
				}
				save_btn.add_css_class("suggested-action");

				save_btn.connect_clicked(clone!(@weak win => move |_| {
					win.close()
				}));

				headerbar.pack_end(&save_btn);
			}
		}

		(win, save_btn)
	}
}
