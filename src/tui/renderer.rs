use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Direction, Constraint, Layout, Rect};
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

// Configuration
/// The colors of the various elements of the TUI.
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Colors {
	/// The color of the decimal separator
    decimal_separator: Color,
	/// The color of the exponent separator (the 'e' at the end of the number)
    exponent_separator: Color,
	/// The color of the digits in the number
    number: Color,
	/// The color of the key labels for memory
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

/// Defines the relative positions of the stack and memory areas.
#[derive(Default, Serialize, Deserialize, Clone, Copy)]
pub enum Orientation {
	/// Stack on the bottom, memory on the top
    StackBottom,
	/// Stack on the top, memory on the bottom
    StackTop,
	/// Stack on the right, memory on the left
    StackRight,
	/// Stack on the left, memory on the right
	#[default]
    StackLeft,
}
// For easy conversion into a layout direction
impl From<Orientation> for Direction {
    fn from(v: Orientation) -> Self {
        match v {
            Orientation::StackBottom | Orientation::StackTop => Direction::Vertical,
            Orientation::StackRight | Orientation::StackLeft => Direction::Horizontal,
        }
    }
}

/// The text alignment of numbers on the stack.
#[derive(Default, Serialize, Deserialize, Clone, Copy)]
pub enum StackAlignment {
	#[default]
	/// Align numbers left
    Left,
	/// Align numbers right
    Right,
}
impl From<StackAlignment> for Alignment {
    fn from(v: StackAlignment) -> Self {
        match v {
            StackAlignment::Left => Alignment::Left,
            StackAlignment::Right => Alignment::Right,
        }
    }
}

/// The text alignment of numbers in memory.
///
/// Due to memory having both a key and the value to worry about, there are three ways to align the
/// memory text. Both the keys and values can be left aligned, the keys can be left and the values
/// right, or both can be right.
#[derive(Default, Serialize, Deserialize, Clone, Copy)]
pub enum MemoryAlignment {
	/// Align keys and numbers left
	AllLeft,
	/// Align keys left and numbers right
	#[default]
	SplitMiddle,
	/// Align keys and numbers right
	AllRight,
}

/// Stores all of the configuration values for the renderer.
#[derive(Default, Serialize, Deserialize)]
pub struct RendererConfig {
    pub colors: Colors,
    pub stack_alignment: StackAlignment,
	pub memory_alignment: MemoryAlignment,
    pub memory_location: Orientation,
}

impl TUI {
	/// Converts an [`f64`] into [`Span`]s representing the number with the styling provided in the
	/// TUI's configuration. This could then be used to insert into a [`Line`], or be further
	/// manipulated. Each contiguous sequence of digits will be one span, and the decimal and
	/// exponent separators will have their own spans dedicated to those characters.
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
            Orientation::StackLeft | Orientation::StackTop => (areas[0], areas[1]),
            Orientation::StackRight | Orientation::StackBottom => (areas[1], areas[0]),
        };

		let main_area_parts: [Rect; 2] =
			Layout::new(
				Direction::Vertical,
				vec![Constraint::Min(self.calc.stack.len() as u16), Constraint::Percentage(100)]
			).areas(main_area);
		let stack_area = main_area_parts[0];
		let command_area = main_area_parts[1];

		// Show error if regions are too small
		// width of 24 is not arbitrary, it permits the full 15 to 17 digits of decimal precision
		// f64 offers as well as allowing room for decimal separator, exponent separator, and
		// exponent digits
		if stack_area.width < 24 || (stack_area.height as usize) < self.calc.stack.len() {
			Paragraph::new("Screen too small!").wrap(Wrap { trim: true }).render(area, buf);
			return;
		}

		// Stack area rendering
        let mut stack_lines: Vec<Line> = vec![];

		// Format each number per the config and insert it into stack_lines
		for stack_value in self.calc.stack.iter().rev() {
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
