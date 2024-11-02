mod input_parser;
mod renderer;

use crossterm;
use crossterm::event;
use crossterm::event::Event;
use crossterm::terminal;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};

use crate::calculator::Calculator;

use input_parser::{ParseAction, Parser};

pub struct TUI {
	calc: Calculator,
	parser: Parser,
	active: bool,
}
impl TUI {
	pub fn new(calc: Calculator) -> TUI {
		TUI {
			calc,
			parser: Parser::new(),
			active: false,
		}
	}
	pub fn run(&mut self) -> std::io::Result<()> {
		let mut out = std::io::stdout();

		terminal::enable_raw_mode()?;
		crossterm::execute!(out, EnterAlternateScreen)?;
		self.active = true;

		let (mut cols, mut rows) = terminal::size()?;
		renderer::draw(self, cols, rows)?;

		'run_loop: loop {
			match event::read()? {
				Event::Key(ke) => {
					let actions = self.parser.parse(ke);
					for action in actions {
						match action {
							ParseAction::Quit => break 'run_loop,
							ParseAction::Command(cmd) => {
								self.calc.process_command(cmd);
							},
						}
					}
				},
				Event::Resize(new_cols, new_rows) => (cols, rows) = (new_cols, new_rows),
				_ => (),
			}

			renderer::draw(self, cols, rows)?;
		}

		crossterm::execute!(out, LeaveAlternateScreen)?;
		terminal::disable_raw_mode()?;
		self.active = false;

		Ok(())
	}
}
// Using `Drop` to ensure the terminal is restored even if a panic or IO error should occur
impl Drop for TUI {
	fn drop(&mut self) {
		if !self.active { return; }

		// Ignore IO errors here, just *try* to reset the settings and if it fails, oh well
		crossterm::execute!(std::io::stdout(), LeaveAlternateScreen).ok();
		terminal::disable_raw_mode().ok();
	}
}
