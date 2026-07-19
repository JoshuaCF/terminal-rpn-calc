use ratatui::buffer::Buffer;
use ratatui::layout::Alignment as RatatuiAlignment;
use ratatui::layout::Direction as RatatuiDirection;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Styled};
use ratatui::text::{Text, Line, Span};
use ratatui::widgets::{Widget, Wrap, Paragraph};
use serde::{Deserialize, Serialize};

use crate::tui::TUI;

/* Things I want to allow customization of regarding rendering:
 * - Decimal separator color
 * - Exponent separator color
 * - Number color
 * - Fixed/flexible decimal separator position
 * - Left align/right align
 * - Precision
 * - Memory key color
 * - Positioning of memory relative to the stack
 */

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Colors {
    decimal_separator: Color,
    exponent_separator: Color,
    number: Color,
    memory_key: Color,
}
impl Default for Colors {
    fn default() -> Self {
        Colors {
            decimal_separator: Color::LightCyan,
            exponent_separator: Color::LightCyan,
            number: Color::Green,
            memory_key: Color::Red,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
impl Default for Direction {
    fn default() -> Self {
        Direction::Right
    }
}
impl From<Direction> for RatatuiDirection {
    fn from(v: Direction) -> Self {
        match v {
            Direction::Up | Direction::Down => RatatuiDirection::Vertical,
            Direction::Left | Direction::Right => RatatuiDirection::Horizontal,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum Alignment {
    Left,
    Right,
}
impl Default for Alignment {
    fn default() -> Self {
        Alignment::Left
    }
}
impl From<Alignment> for RatatuiAlignment {
    fn from(v: Alignment) -> Self {
        match v {
            Alignment::Left => RatatuiAlignment::Left,
            Alignment::Right => RatatuiAlignment::Right,
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct RendererConfig {
    colors: Colors,
    alignment: Alignment,
    memory_location: Direction,
}

impl TUI {
	fn style_number<'a>(&self, number: f64, width: u16) -> Vec<Span<'a>> {
		// First try doing a plain format
		// If too long, then try exponential with precision of WIDTH-3 (decimal separator, e, and
		// digit)
		// Continue reducing precision until it fits
		// Maybe there's a smarter way, but this is plenty good enough
		let mut num_string = format!("{:>1$}", number, width as usize);
		// .len() acceptable here since number formatting will only use ASCII which is 1 char to
		// 1 byte
		let mut precision = (width - 3) as usize;
		while num_string.len() > width as usize {
			num_string = format!("{:.precision$e}", number);

			if precision == 0 {
				// can't see how this could happen but just in case
				// TODO: handle this better
				return vec![Span::from("Render err")];
			}
			precision -= 1;
		}

		// Style the parts of the number
		// numbers[decimal numbers[exponent numbers]]
		let mut cur_spans = vec![];

		let exponent_idx = num_string.find('e');
		let decimal_idx = num_string.find('.');
		let mut cur_idx = 0;

		if let Some(to) = decimal_idx {
			cur_spans.push(String::from(&num_string[cur_idx..to]).set_style(self.config.renderer.colors.number));
			cur_spans.push(String::from(&num_string[to..to+1]).set_style(self.config.renderer.colors.decimal_separator));
			cur_idx = to + 1; // Again, working with only ASCII so this is okay
		}
		if let Some(to) = exponent_idx {
			cur_spans.push(String::from(&num_string[cur_idx..to]).set_style(self.config.renderer.colors.number));
			cur_spans.push(String::from(&num_string[to..to+1]).set_style(self.config.renderer.colors.exponent_separator));
			cur_idx = to + 1; // See above
		}
		cur_spans.push(String::from(&num_string[cur_idx..]).set_style(self.config.renderer.colors.number));
		cur_spans
	}
}

impl Widget for &TUI {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // TODO: Maybe let this be configurable too?
        let constraints: Vec<Constraint> =
            vec![Constraint::Percentage(50), Constraint::Percentage(50)];

        let layout = Layout::new(self.config.renderer.memory_location.into(), constraints);

        let areas: [Rect; 2] = layout.areas(area);
        let (main_area, memory_area) = match self.config.renderer.memory_location {
            Direction::Right | Direction::Down => (areas[0], areas[1]),
            Direction::Left | Direction::Up => (areas[1], areas[0]),
        };

		let main_area_parts: [Rect; 2] =
			Layout::new(
				RatatuiDirection::Vertical,
				vec![Constraint::Min(self.calc.nums.len() as u16), Constraint::Percentage(100)]
			).areas(main_area);
		let stack_area = main_area_parts[0];
		let command_area = main_area_parts[1];

		// Show error if regions are too small
		// width of 24 is not arbitrary, it permits the full 15 to 17 digits of decimal precision
		// f64 offers as well as allowing room for decimal separator, exponent separator, and
		// exponent digits
		if stack_area.width < 24 || (stack_area.height as usize) < self.calc.nums.len() {
			Paragraph::new("Screen too small!").wrap(Wrap { trim: true }).render(area, buf);
			return;
		}

		// Stack area rendering
        let mut stack_lines: Vec<Line> = vec![];

		// Format each number per the config and insert it into stack_lines
		for stack_value in self.calc.nums.iter().rev() {
			stack_lines.push(Line::from(self.style_number(*stack_value, stack_area.width)));
		}
		Text::from(stack_lines).render(stack_area, buf);

		// Memory area rendering
		let mut memory_lines: Vec<Line> = vec![];
		for (key, val) in self.calc.memory.iter() {
			let mut cur_line = Line::default();
			let mut memory_prefix = key.to_string();
			memory_prefix.push_str(": ");

			let line_width = memory_area.width - memory_prefix.len() as u16;
			cur_line.push_span(memory_prefix.set_style(self.config.renderer.colors.memory_key));

			for span in self.style_number(*val, line_width) {
				cur_line.push_span(span);
			}
			memory_lines.push(cur_line);
		}
		Text::from(memory_lines).render(memory_area, buf);

		// Command area rendering
		let mut cur_command = String::new();
		for cur_char in &self.parser.bfr {
			cur_command.push(*cur_char);
		}
		cur_command.render(command_area, buf);
    }
}
