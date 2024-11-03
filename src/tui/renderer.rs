use std::io::Write;

use crossterm::queue;
use crossterm::cursor::*;
use crossterm::style::*;
use crossterm::terminal::*;

use crate::tui::TUI;

impl TUI {
	pub fn draw(&self, cols: u16, rows: u16) -> std::io::Result<()> {
		// NOTE: This is a temporary renderer
		let mut out = std::io::stdout();

		queue!(out, MoveTo(0, 0))?;

		let mut memory_pairs = self.calc.memory.iter().collect::<Vec<(&char, &f64)>>();
		memory_pairs.sort_by(|l, r| l.0.cmp(r.0));

		for (k, v) in memory_pairs {
			queue!(out, Print(format!("{} : {:<015.7}", k, v)), Clear(ClearType::UntilNewLine), MoveToNextLine(1))?;
		}

		queue!(out, MoveToNextLine(1))?;

		for i in (0..self.calc.nums.len()).rev() {
			queue!(out, Print(format!("{:<015.7}", self.calc.nums[i])), Clear(ClearType::UntilNewLine), MoveToNextLine(1))?;
		}
		queue!(out, Print(self.parser.bfr.iter().collect::<String>()), Clear(ClearType::UntilNewLine))?;

		out.flush()?;
		Ok(())
	}
}
