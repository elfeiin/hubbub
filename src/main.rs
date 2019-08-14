extern crate ws;
use ws::{listen, Handler, Sender, Result, Message, Handshake, CloseCode, Error};
use serde::{Deserialize, Serialize};
use serde_json::Result as jsResult;
use std::rc::Rc;
use std::cell::RefCell;
extern crate chrono;
use chrono::prelude::*;
use std::fs::File;
use std::io::prelude::*;

fn main() {
	let mut users: Rc<RefCell<Vec<User>>> = Rc::new(RefCell::new(vec!()));
	let mut conversations: Rc<RefCell<Vec<Rc<RefCell<Conversation>>>>> = Rc::new(RefCell::new(vec!()));
	listen("0.0.0.0:30012", |out| Server { out: out, users: users.clone(), conversations: conversations.clone() }).unwrap();
}

struct Server {
	out: Sender,
	users: Rc<RefCell<Vec<User>>>,
	conversations: Rc<RefCell<Vec<Rc<RefCell<Conversation>>>>>,
}

//Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
impl Handler for Server {
	
	fn on_open(&mut self, hs: ws::Handshake) -> Result<()> {
		println!("Client connection established.");
		println!("{:?}", self.out.token());
		// self.out.send(self.count.get());
		Ok(())
	}
	
	fn on_message(&mut self, msg: Message) -> Result<()> {
		// self.out.send(format!("{{ \"text\": \"{}\" }}", msg));
		let decoded: Demessage = serde_json::from_str(&msg.to_string()).unwrap();
		self.handle_msg(decoded);
		// self.out.broadcast(msg);
		Ok(())
	}
	
	fn on_close(&mut self, code: CloseCode, reason: &str) {
		self.remove_user(self.out.token());
		match code {
			CloseCode::Normal => println!("The client is done with the connection."),
			CloseCode::Away => println!("The client is leaving the site."),
			CloseCode::Abnormal => println!("Closing handshake failed. Unable to obtain closing status from client."),
			_ => println!("The client encountered an error (CloseCode: {:?}): {}", code, reason),
		}
	}
	
	fn on_error(&mut self, err: Error) {
		println!("The server encountered an error: {:?}", err);
	}
	
}

impl Server {
	
	pub fn validate(self, ) -> Option<()> {
		
		Some(())
	}
	
	pub fn add_convo(&mut self, id: usize, owner: usize) {
		let mut convo = Rc::new(RefCell::new(Conversation {
			id: id,
			buffer: String::new(),
			owner: owner,
			admins: vec!(),
			users: vec!(),
			banned: vec!(),
			private: true,
		}));
		self.conversations.borrow_mut().push(convo);
	}
	
	pub fn add_user(&mut self, usrid: usize, token: ws::util::Token) {
		
	}
	
	pub fn remove_user(&mut self, token: ws::util::Token) {
		for (i, u) in self.users.borrow_mut().iter_mut().enumerate() {
			if u.token == token {
				self.users.borrow_mut().remove(i);
				break;
			}
		}
	}
	
	pub fn id_get_user_token(&self, id: &usize) -> Option<ws::util::Token> {
		self.users.borrow().iter().find(|u| &u.id == id).map(|u| u.token)
	}
	
	fn get_convo_mut(&self, convoid: &usize) -> Option<Rc<RefCell<Conversation>>> {
		self.conversations.borrow().iter().find(|c| c.borrow().id() == convoid).map(|c| c.clone())
	}
	
	fn get_users_from_convo(&self, convoid: &usize) -> Option<Vec<usize>> {
		self.conversations.borrow().iter().find(|c| c.borrow().id() == convoid).map(|c| c.borrow().users.clone())
	}
	
	pub fn has_permission(&self, convoid: &usize, userid: &usize) -> bool {
		self.conversations.borrow().iter().find(|c| c.borrow().id() == convoid).map(|c| c.borrow().has_permission(userid)).unwrap()
	}
	
	fn send_to_all_in_convo(&self, convoid: &usize, exclude: &usize) {
		if let Some(users) = self.get_users_from_convo(convoid) {
			for id in users.iter() {
				if id != exclude && self.has_permission(convoid, id) {
					if let Some(token) = self.id_get_user_token(id) {
						
					}
				}
			}
		}
	}
	
	fn handle_msg(&mut self, msg: Demessage) -> Result<()> {
		match &msg.cmd[..] {
			"Replace" => {
				if let Some(mut convo) = self.get_convo_mut(&msg.convo) {
					if let Some(first) = convo.borrow().buffer().get(0..msg.start) {
						if let Some(third) = convo.borrow().buffer().get(msg.end..convo.borrow().buffer().len()) {
							let whole = format!("{}{}{}", first, msg.txt, third);
							convo.borrow_mut().set_buffer(&msg.usrid, whole);
						}
					}
				}
			},
			"MoveCursor" => {
				self.send_to_all_in_convo(&msg.convo, &msg.usrid);
			},
			"RequestConvo" => {
				
			},
			"NewUser" => {
				self.add_user(msg.usrid, self.out.token());
			},
			_ => (),
		}
		Ok(())
	}
	
}

#[derive(Debug, PartialEq, Eq)]
struct User {
	id: usize,
	token: ws::util::Token,
}

impl User {
	pub fn new(id: usize, token: ws::util::Token) -> User {
		User {
			id: id,
			token: token,
		}
	}
	
	pub fn id(&self) -> &usize {
		&self.id
	}
	
	pub fn token(&self) -> &ws::util::Token {
		&self.token
	}
	
}

#[derive(PartialEq)]
enum Permissions {
	Owner,
	Admin,
	User,
	Banned,
	Apart,
}

struct Conversation {
	id: usize,
	buffer: String,
	owner: usize,
	admins: Vec<usize>,
	users: Vec<usize>,
	banned: Vec<usize>,
	private: bool,
}

impl Conversation {
	
	pub fn has_permission(&self, userid: &usize) -> bool {
		if let Some(p) = self.get_user_permissions(userid) {
			if p != Permissions::Banned && p != Permissions::Apart {
				return true;
			}
		}
		false
	}
	
	pub fn get_user_permissions(&self, userid: &usize) -> Option<Permissions> {
		if self.owner() == userid {
			return Some(Permissions::Owner);
		}
		for i in self.banned.iter() {
			if i == userid {
				return Some(Permissions::Banned);
			}
		}
		for i in self.admins.iter() {
			if i == userid {
				return Some(Permissions::Admin);
			}
		}
		for i in self.users.iter() {
			if i == userid {
				return Some(Permissions::User);
			}
		}
		Some(Permissions::Apart)
	}
	
	pub fn id(&self) -> &usize {
		&self.id
	}
	
	pub fn set_buffer(&mut self, usrid: &usize, s: String) {
		let p = self.get_user_permissions(usrid).unwrap();
		if p != Permissions::Apart && p != Permissions::Banned {
			self.buffer = s;
		}
	}
	
	pub fn buffer(&self) -> &String {
		&self.buffer
	}
	
	pub fn set_owner(&mut self, usrid: usize) {
		self.owner = usrid;
	}
	
	pub fn owner(&self) -> &usize {
		&self.owner
	}
	
	pub fn add_admin(&mut self, usrid: usize) {
		self.admins.push(usrid);
	}
	
	pub fn admins(&self) -> &Vec<usize> {
		&self.admins
	}
	
	pub fn add_user(&mut self, usrid: usize) {
		self.users.push(usrid);
	}
	
	pub fn users(&self) -> &Vec<usize> {
		&self.users
	}
	
	pub fn ban(&mut self, usrid: usize) {
		self.banned.push(usrid);
	}
	
	pub fn banned(&self) -> &Vec<usize> {
		&self.banned
	}
	
	pub fn toggle_private(&mut self, b: bool) {
		self.private = b;
	}
	
	pub fn private(&self) -> &bool {
		&self.private
	}
	
}

#[derive(Serialize, Deserialize)]
struct Demessage {
	usrid: usize,
	convo: usize,
	cmd: String,
	txt: String,
	start: usize,
	end: usize,
}