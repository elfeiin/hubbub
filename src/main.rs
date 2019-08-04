extern crate ws;
use ws::{listen, Handler, Sender, Result, Message, Handshake, CloseCode, Error};
extern crate chrono;
use chrono::prelude::*;
use std::rc::Rc;
use std::cell::Cell;
extern crate rustc_serialize;
use rustc_serialize::json::Json;

//Local::now().format("%Y-%m-%d %H:%M:%S").to_string()

struct Server<'a> {
	out: Sender,
	count: Rc<Cell<usize>>,
	hubbub: &'a Hubbub,
}

impl<'a> Handler for Server<'a> {
	
	fn on_open(&mut self, hs: ws::Handshake) -> Result<()> {
		println!("Client connection established.");
		println!("{:?}", self.out.token());
		Ok(self.count.set(self.count.get() + 1))
	}
	
	fn on_message(&mut self, msg: Message) -> Result<()> {
		if let Some(data) = Json::from_str(&msg.to_string()).unwrap().as_object() {
			if let Some(convo) = data.get("convo").unwrap().as_u64() {
				if let Some(convo) = &self.hubbub.get_convo(convo as usize) {
					let buff = &convo.buffer;
					if let Some(cmd) = data.get("cmd").unwrap().as_string() {
						if let Some(txt) = data.get("txt").unwrap().as_string() {
							if let Some(start) = data.get("start").unwrap().as_u64() {
								let start = start as usize;
								if let Some(end) = data.get("end").unwrap().as_u64() {
									let end = end as usize;
									match cmd {
										"Replace" => {
											if let Some(first) = buff.get(0..start) {
												if let Some(third) = buff.get(end..buff.len()) {
													let whole = format!("{}{}{}", first, txt, third);
													// self.buff = whole;
												}
											}
										},
										"MoveCursor" => {
										},
										"RequestConvo" => {
											
										}
										_ => (),
									}
								}
							}
						}
					}
				}
			}
		}
		// self.out.send(format!("{{ \"text\": \"{}\" }}", msg))
		self.out.close(CloseCode::Normal);
		Ok(())
	}
	
	fn on_close(&mut self, code: CloseCode, reason: &str) {
		match code {
			CloseCode::Normal => println!("The client is done with the connection."),
			CloseCode::Away => println!("The client is leaving the site."),
			CloseCode::Abnormal => println!("Closing handshake failed. Unable to obtain closing status from client."),
			_ => println!("The client encountered an error (CloseCode: {:?}): {}", code, reason),
		}
		
		self.count.set(self.count.get() - 1)
	}
	
	fn on_error(&mut self, err: Error) {
		println!("The server encountered an error: {:?}", err);
	}
	
}

fn main() {
	let count = Rc::new(Cell::new(0));
	let mut h = Hubbub::new();
	listen("0.0.0.0:30012", |out| Server { out: out, count: count.clone(), hubbub: &mut h, }).unwrap();
	println!("{:?}", count);
}

struct User {
	id: usize,
	convos: Vec<usize>,
	owns: Vec<usize>,
	administrates: Vec<usize>,
}

impl User {
	pub fn new(id: usize) -> User {
		User {
			id: id,
			convos: vec!(),
			owns: vec!(),
			administrates: vec!(),
		}
	}
	
	pub fn add_to_convo(&mut self, id: usize) -> std::result::Result<(), String> {
		let mut convofound = false;
		self.convos.iter().map(|c| { if c == &id { convofound = true } });
		if !convofound {
			self.convos.push(id);
			Ok(())
		} else {
			Err(String::from("User already in convo"))
		}
	}
}

struct Conversation {
	id: usize,
	buffer: String,
	private: bool,
}

struct Hubbub {
	users: Vec<User>,
	conversations: Vec<Conversation>,
}

impl Hubbub {
	
	pub fn new() -> Hubbub {
		Hubbub {
			users: vec!(),
			conversations: vec!(),
		}
	}
	
	pub fn add_convo(&mut self, id: usize, owner: usize) {
		if let Some(usr) = self.get_user(owner) {
			let mut convo = Conversation {
				id: id,
				buffer: String::new(),
				private: true,
			};
			usr.add_to_convo(id);
			self.conversations.push(convo);
		}
	}
	
	fn get_convo(&mut self, id: usize) -> Option<&mut Conversation> {
		for c in self.conversations.iter_mut() {
			if c.id == id {
				return Some(c);
			}
		}
		None
	}
	
	
	pub fn get_user(&mut self, id: usize) -> Option<&mut User> {
		for mut u in self.users.iter_mut() {
			if u.id == id {
				return Some(u);
			}
		}
		None
	}
	fn send_to_all_in_convo(&mut self, convo_id: usize) -> Result<()> {
		Ok(())
	}
	
}
