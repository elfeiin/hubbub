extern crate ws;
use ws::{listen, Handler, Sender, Result, Message, Handshake, CloseCode, Error};
extern crate chrono;
use chrono::prelude::*;
use std::rc::Rc;
use std::cell::Cell;

struct Server {
	out: Sender,
	count: Rc<Cell<u32>>,
}

impl Handler for Server {
	
	fn on_open(&mut self, hs: ws::Handshake) -> Result<()> {
		Ok(self.count.set(self.count.get() + 1))
	}
	
	fn on_message(&mut self, msg: Message) -> Result<()> {
		println!("{:?}", msg);
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

struct UserMessage {
	senderid: u32,
	text: String,
	timestamp: DateTime<Local>,
}

struct User {
	id: u32,
	buffer: String,
}

struct Conversation {
	users: Vec<User>,
	owner: u32,
	log: Vec<UserMessage>,
}

struct Hubbub {
	conversations: Vec<Conversation>,
}

impl Hubbub {
	pub fn add_convo(&mut self, owner: u32) {
		let mut convo = Conversation {
			users: vec!(),
			owner: owner,
			log: vec!(),
		};
		convo.add_user(owner);
		self.conversations.push(convo);
	}
}

impl Conversation {
	pub fn add_user(&mut self, id: u32) -> std::result::Result<(), String> {
		let mut userfound = false;
		self.users.iter().map(|u| { if u.id == id { userfound = true } });
		if !userfound {
			self.users.push(
				User {
					id: id,
					buffer: String::new(),
				}
			);
			Ok(())
		} else {
			Err(String::from("User already in convo"))
		}
	}
}

fn main() {
	let count = Rc::new(Cell::new(0));
	listen("0.0.0.0:30012", |out| { Server { out: out, count: count.clone() } }).unwrap()
}