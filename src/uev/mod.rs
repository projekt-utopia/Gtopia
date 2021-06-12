mod stream;

use crate::config::APP_ID;

use utopia_common::frontend as utopia;

use tokio::{runtime::Runtime, io::{AsyncReadExt, AsyncWriteExt}};
use futures::{stream::StreamExt, channel::mpsc};

use libadwaita::prelude::*;
//use gtk::prelude::*;

use gtk::glib::{Receiver, Sender, MainContext, PRIORITY_DEFAULT};

use std::thread;
use std::sync::{Arc, Mutex};

macro_rules! send {
    ($sender:expr, $msg:expr) => {
        if let Err(e) = $sender.send($msg) {
            eprintln!("Channel died, closing loop: {}", e);
            return;
        }
    }
}

#[derive(Debug)]
pub enum UtopiaRequest {
	GetGameLibrary,
	GetFullGameLibrary,
	//GetGameDetails(String /* uuid */),
	TriggerLaunch(String /* uuid */),
	// uuid of game, uuid of provider
	TriggerProviderUpdate(String, String)
}

#[derive(Debug)]
pub enum UtopiaMessage {
	RefreshGameLibrary(Vec<utopia::library::LibraryItemFrontendDetails>),
	Disconnect
}

#[derive(Debug)]
pub struct UtopiaEvents {
	sender: Sender<UtopiaMessage>,
	pub channel: mpsc::Sender<UtopiaRequest>,
	receiver: Arc<Mutex<mpsc::Receiver<UtopiaRequest>>>
}

impl UtopiaEvents {
	pub fn new() -> (Self, mpsc::Sender<UtopiaRequest>, Receiver<UtopiaMessage>) {
		let (tcx, rcx) = mpsc::channel(0xF);
		let (tx, rx) = MainContext::channel(PRIORITY_DEFAULT);
		(Self {
			sender: tx,
			channel: tcx.clone(),
			receiver: Arc::new(Mutex::new(rcx))
		}, tcx, rx)
	}

	pub fn start(&self) {
		let sender = self.sender.clone();
		let receiver = self.receiver.clone();
		thread::spawn(move || {
			let rt = Runtime::new().unwrap();
			rt.block_on(async {
				let stream = match tokio::net::UnixStream::connect(
					format!("{}/utopia.sock", std::env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR was not set")))
					.await {
						Ok(sock) => sock,
						Err(e) => {
							println!("Unable to connect to µtopia daemon: {}", e);
							send!(sender, UtopiaMessage::Disconnect);
							return
						}
					};
				stream.ready(tokio::io::Interest::READABLE | tokio::io::Interest::WRITABLE).await.unwrap();
				let mut socket = stream::SocketStream::from_stream(stream);
				if let Err(e) = socket.write(APP_ID.as_bytes()).await {
					eprintln!("Error duing handshake with µtopia: {}", e);
					send!(sender, UtopiaMessage::Disconnect);
					return;
				} else {
					let mut buf = [0; 0xFF];
					// TODO: implement timeout
					let n = socket.read(&mut buf).await.unwrap();
					match serde_json::from_slice::<utopia::CoreEvent>(&buf.split_at(n).0) {
						Ok(hs) => println!("Sucessfull handshake: {:?}", hs),
						Err(e) => {
							eprintln!("Error duing handshake with µtopia: {}", e);
							send!(sender, UtopiaMessage::Disconnect);
							return;
						}
					}
				}

				let mut receiver = receiver.try_lock().unwrap();
				loop {
					futures::select! {
						ev = socket.next() => {
							if let Some(ev) = ev {
								match ev {
									Ok(ev) => {
										match ev.action {
											utopia::CoreActions::ResponseFullGameLibrary(library) => {
												send!(sender, UtopiaMessage::RefreshGameLibrary(library))
											},
											_ => println!("Something else: {:?}", ev.action)
										}
									},
									Err(e) => eprintln!("Error getting message from µtopia: {}", e)
								}
							}
						},
						req = receiver.next() => {
							if let Some(req) = req {
								match req {
									UtopiaRequest::GetFullGameLibrary => {
										let library_reqw = utopia::FrontendEvent {
											version: String::from("0.0.0"),
											uuid: Some(String::from(crate::config::APP_ID)),
											action: utopia::FrontendActions::GetFullGameLibrary
										};
										//socket.block_writeable().await.unwrap();
										tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
										socket.write_s(&serde_json::to_vec(&library_reqw).unwrap()).await.unwrap();
									},
									UtopiaRequest::TriggerLaunch(uuid) => {
										let library_reqw = utopia::FrontendEvent {
											version: String::from("0.0.0"),
											uuid: Some(String::from(crate::config::APP_ID)),
											action: utopia::FrontendActions::GameMethod(utopia_common::library::LibraryItemProviderMethods::Launch(uuid))
										};
										//socket.block_writeable().await.unwrap();
										//tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
										socket.write_s(&serde_json::to_vec(&library_reqw).unwrap()).await.unwrap();
									},
									UtopiaRequest::TriggerProviderUpdate(uuid, provider) => {
										let library_reqw = utopia::FrontendEvent {
											version: String::from("0.0.0"),
											uuid: Some(String::from(crate::config::APP_ID)),
											action: utopia::FrontendActions::GameMethod(utopia_common::library::LibraryItemProviderMethods::ChangeSelectedProvider(uuid, provider))
										};
										//socket.block_writeable().await.unwrap();
										//tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
										socket.write_s(&serde_json::to_vec(&library_reqw).unwrap()).await.unwrap();
									},
									_ => eprintln!("something else: {:?}", req)
								}
							}
						}
					}
				}
			});
		});
	}

	pub fn request_library(&mut self) {
		self.channel.try_send(UtopiaRequest::GetFullGameLibrary).unwrap();
	}
}

pub fn handle_event(event: UtopiaMessage, channel: mpsc::Sender<UtopiaRequest>, window: crate::utopia::UtopiaWindow) -> gtk::glib::Continue {
	println!("New msg: {:?}", event);
	match event {
		UtopiaMessage::Disconnect => {
			let label = gtk::LabelBuilder::new()
				.label("Failed communicating with the µtopia daemon, please restart.")
				.hexpand(true)
				.build();
			let container = gtk::BoxBuilder::new()
				.orientation(gtk::Orientation::Vertical)
				.build();
			gtk::prelude::BoxExt::append(&container, &label);
			window.set_child(Some(&container));
			return gtk::glib::Continue(false);
		},
		UtopiaMessage::RefreshGameLibrary(library) => {
			//println!("Library: {:#?}", library);
			for item in library {
				//channel.try_send(UtopiaRequest::GetGameDetails(item.uuid)).unwrap();
				//println!("Item: {:?}", item);
				window.new_item(item);
			}
		}
	};
	gtk::glib::Continue(true)
}
