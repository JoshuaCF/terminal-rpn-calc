use std::io::Write;

use crossterm::queue;
use crossterm::cursor::*;
use crossterm::style::*;
use crossterm::terminal::*;

use crate::tui::TUI;

pub fn draw(app: &TUI, cols: u16, rows: u16) -> std::io::Result<()> {
	// NOTE: This is a temporary renderer
	let mut out = std::io::stdout();

	queue!(out, MoveTo(0, 0))?;

	for i in (0..app.calc.nums.len()).rev() {
		queue!(out, Print(format!("{}", app.calc.nums[i])), Clear(ClearType::UntilNewLine), MoveToNextLine(1))?;
	}
	queue!(out, Print(app.parser.bfr.iter().collect::<String>()), Clear(ClearType::UntilNewLine))?;

	out.flush()?;
	Ok(())
}
