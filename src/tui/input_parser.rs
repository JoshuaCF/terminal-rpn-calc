use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::calculator::{CalcCmd, Calculator};

pub enum ParseAction {
	Quit,
	Command(CalcCmd),
}

pub struct Parser {
	pub bfr: Vec<char>
}
impl Parser {
	pub fn new() -> Parser {
		Parser {
			bfr: Vec::new(),
		}
	}

	// When pressing a button with an immediate action, the current buffer is evaluated to
	// determine if an action should occur before the immediate action, which can cause
	// more than once action to be performed in a single press, hence the need for `Vec`

	// Maybe return an `Option` rather than an empty `Vec`?
	pub fn parse(&mut self, ke: KeyEvent) -> Vec<ParseAction> {
		let mut actions = Vec::new();

		// I don't know a better way to return early on non-presses
		match ke.kind {
			KeyEventKind::Press => (),
			_ => return actions
		}

		// TODO: Make the bindings configurable
		match ke.code {
			KeyCode::Backspace => { self.bfr.pop(); },
			KeyCode::Enter => {
				if let Some(a) = self.decode_bfr() {
					actions.push(a);
				}
			},
			KeyCode::Left => {},
			KeyCode::Right => {},
			KeyCode::Up => {},
			KeyCode::Down => {},
			KeyCode::Home => {},
			KeyCode::End => {},
			KeyCode::PageUp => {},
			KeyCode::PageDown => {},
			KeyCode::Tab => {},
			KeyCode::BackTab => {},
			KeyCode::Delete => { self.bfr.pop(); },
			KeyCode::Insert => {},
			KeyCode::F(num) => {},
			KeyCode::Char(c) => if c.is_alphanumeric() { self.bfr.push(c); },
			KeyCode::Esc => {},
			_ => (),
		}

		actions
	}

	fn decode_bfr(&mut self) -> Option<ParseAction> {
		let mut clear_bfr = true;

		let action = match self.bfr.iter().collect::<String>().as_str() {
			"quit" => Some(ParseAction::Quit),
			_ => {
				clear_bfr = false;
				None
			},
		};

		if clear_bfr { self.bfr.clear(); }

		action
	}
}
