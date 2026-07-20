mod input_parser;
mod renderer;

use crossterm::event;
use crossterm::event::Event;

use ratatui;

use serde::{Deserialize, Serialize};

use crate::calculator::Calculator;

use input_parser::{ExternalCommand, Parser, ParserConfig};
use renderer::RendererConfig;

// Configuration
#[derive(Default, Serialize, Deserialize)]
pub struct TUIConfig {
	renderer: RendererConfig,
	parser: ParserConfig,
}

pub struct TUI {
	calc: Calculator,
	parser: Parser,
	config: TUIConfig,
}
impl TUI {
	pub fn new(calc: Calculator, config: TUIConfig) -> TUI {
		TUI {
			calc,
			parser: Parser::new(),
			config,
		}
	}
	pub fn run(&mut self) -> std::io::Result<()> {
		let mut terminal = ratatui::init();

		'run_loop: loop {
			terminal.draw(|frame| frame.render_widget(&*self, frame.area()))?;

			match event::read()? {
				Event::Key(ke) => {
					let actions = self.parser.parse(ke);
					for action in actions {
						match action {
							ExternalCommand::Quit => break 'run_loop,
							ExternalCommand::CalcCmd(cmd) => {
								self.calc.process_command(cmd);
							},
						}
					}
				},
				_ => (),
			}
		}

		ratatui::restore();

		Ok(())
	}
}
