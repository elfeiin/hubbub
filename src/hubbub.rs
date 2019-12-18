use termion::{color, cursor, style};

const NICK_MAX_LEN: usize = 32;
const NICK_MIN_LEN: usize = 0;

pub struct Workspace {
	messages: Vec<Message>, // List of received messages
	buffers: Vec<Message>, // List of currently being typed messages
}

impl Workspace {
	pub fn change(&mut self, change: Change) { // Takes a Change struct and processes it
		let mut m_index: Option<usize> = None;
		for (i, m) in self.buffers.iter().enumerate() {
			if m.sender_ip == change.sender_ip {
				m_index = Some(i);
				break;
			}
		}
		
		if let Some(i) = m_index {
			self.process_action(i, change);
			return;
		}
		
		let mut m = Message {
			sender_ip: change.sender_ip,
			sender_nick: change.sender_nick.clone(),
			text: String::new(),
		};
		self.buffers.push(m);
		self.process_action(self.buffers.len()-1, change);
	}
	
	fn process_action(&mut self, m_index: usize, change: Change) {
		let mut message = &mut self.buffers[m_index];
		
		message.sender_nick = change.sender_nick;
		
		use Action::*;
		match change.action {
			Replace(s, t) => {
				let len = message.text.len();
				self.replace(m_index, s.clamp(0, len), t);
			}
			Solidify => self.solidify(m_index),
		}
	}
	
	fn solidify(&mut self, m_index: usize) {
		let mut message = &mut self.buffers[m_index];
		
		self.messages.push(message.clone());
		
		message.text = String::new();
		
	}
	
	fn replace(&mut self, m_index: usize, selection: Selection, text: String) {
		self.buffers[m_index].text.replace_range(selection.to_range(), text.as_ref());
	}
}

#[derive(Clone)]
pub struct Message {
	sender_ip: u128,
	sender_nick: Option<Nick>,
	text: String,
}

impl core::fmt::Display for Message {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> core::fmt::Result {
		if let Nick::Some(nick) = &self.sender_nick {
			write!(f, "\n{}", nick)?;
		}
		write!(f, "@{}", self.sender_ip.to_ipv6())?;
		write!(f, "\n\t{}", self.text)?;
		Ok(())
	}
}

#[derive(Clone)]
pub struct Change {
	sender_ip: u128,
	sender_nick: Nick,
	action: Action,
}

#[derive(Clone)]
pub enum Action {
	Replace(Selection, String),
	Solidify,
}

#[derive(Clone)]
pub struct Selection {
	start: usize,
	end: usize,
}

impl Selection {
	pub fn to_range(&self) -> std::ops::Range<usize> {
		self.start.min(self.end)..self.start.max(self.end)
	}
	
	pub fn clamp(mut self, start: usize, end: usize) -> Self {
		self.start = self.start.min(end).max(start);
		self.end = self.end.min(end).max(start);
		self
	}
}

#[derive(Clone)]
pub enum Nick {
  Some(String),
  None
};

impl Nick {
	pub fn new(name: String) -> Result<Self, String> {
		let graphemes = name.chars().count();
		
		if graphemes > NICK_MIN_LEN && graphemes < NICK_MAX_LEN {
			Ok(Nick(name))
		} else {
			Err(format!("Nickname should be between {} and {} grapheme clusters.", NICK_MIN_LEN, NICK_MAX_LEN))
		}
	}
}

trait IsIPv6 {
	fn to_ipv6(&self) -> String;
}

impl IsIPv6 for u128 {
	fn to_ipv6(&self) -> String {
		let mut ip = String::new();
		let bytes = self.to_le_bytes();
		for (i, x) in bytes.iter().enumerate().step_by(2) {
			if *x > 0 {
				ip = format!("{}{:X}{:X}", ip, x, bytes[i+1]);
			}
			if i < 14 {
				ip = format!("{}:", ip);
			}
		}
		ip
	}
}

#[cfg(test)]
mod tests {
	
	#[test]
	fn replace_works() {
		
		let mut ws = crate::hubbub::Workspace {
			messages: vec!(),
			buffers: vec!(crate::hubbub::Message {
				sender_ip: 0,
				sender_nick: None,
				text: String::new(),
			}),
		};
		
		ws.change(crate::hubbub::Change {
			sender_ip: 0,
			sender_nick: None,
			action: crate::hubbub::Action::Replace(
				crate::hubbub::Selection {start: 4, end: 8},
				String::from("Ohhhh MAN! X3")
			)
		});
		
		assert_eq!(ws.buffers[0].text, String::from("Ohhhh MAN! X3"));
		
		ws.change(crate::hubbub::Change {
			sender_ip: 0,
			sender_nick: None,
			action: crate::hubbub::Action::Replace(
				crate::hubbub::Selection {start: 4, end: 8},
				String::from("")
			)
		});
		assert_eq!(ws.buffers[0].text, String::from("OhhhN! X3"));
		
	}
	
}