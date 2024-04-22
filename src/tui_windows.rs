/*
How do I want to approach this?
Desired behavior: ability to create panes that can be composed into groups with relative sizes and easily resized through that
	- These panes can be of different types and have different functionalities
	- A main controller struct should be able to contain the configuration of the pane's rendering and the absolute sizes
		the panes have to work within should be computed automatically by this module
	- The main controller struct will still be in charge of handling user input and sending the events to the appropriate panes
	- The main controller struct should be able to be aware of the actual type of the pane and call all its unique functions

For implementation, one of the `Cell` types might work well here.
Structs that are meant to be displayed in a pane should implement a trait for the display.
	- The actual display should not be direct, instead it should return the list of actions it would like to take
	- These actions can be enum variants
	- The actual code for rendering it will be a struct that defines a pane's configuration (line wrapping, border, margin, etc.)
	- This struct will contain a trait object of the thing it actually tries to render
		- This pane configuration will also implement the trait so that the panes can be nested!
	- When the pane configuration is rendered, it will compute the areas available for its children and pass them that information
	- It will then take the returned actions, apply the configurations to them (wrapping when the line goes off the edge if enabled, etc.)
		and render appropriately
*/

use crossterm::queue;
use crossterm::cursor::*;
use crossterm::style::*;

use std::cell::RefCell;
use std::io::Write;

#[derive(Debug)]
pub enum StyleProperty {
	FgColor(Color),
	BgColor(Color),
}

#[derive(Debug)]
pub struct PrettyString {
	contents: String,
	style: Vec<StyleProperty>,
}
impl PrettyString {
	pub fn new(contents: String) -> PrettyString {
		PrettyString { contents, style: vec!() }
	}
	pub fn style(mut self, style: StyleProperty) -> Self {
		self.style.push(style);
		self
	}
}

// Mostly wrappers around crossterm's commands
#[derive(Debug)]
pub enum RenderAction {
	MoveTo(u16, u16),
	MoveToNextLine(u16),
	ClearToNextLine,
	ClearToEnd,
	Write(PrettyString),
	HideCursor,
	ShowCursor,
}

pub trait WindowDisplay {
	fn render(&self, size: (u16, u16)) -> Vec<RenderAction>;
}

#[derive(Clone, Copy)]
pub struct WindowConfig {
	pub rel_size: f64,
	pub wrapping: bool,
}
impl Default for WindowConfig {
	fn default() -> WindowConfig {
		WindowConfig { rel_size: 1.0, wrapping: false }
	}
}

pub enum TileType<'a> {
	Window(Window<'a>),
	Container(Container<'a>),
}

pub enum ContainerOrientation {
	Vertical,
	Horizontal,
}
pub struct Container<'a> {
	pub tiles: Vec<TileType<'a>>,
	pub orientation: ContainerOrientation,
	pub rel_size: f64,
}
impl<'a> Container<'a> {
	pub fn new(tiles: Vec<TileType<'a>>, vertical: bool, rel_size: f64) -> Container<'a> {
		Container {
			tiles,
			orientation: match vertical {
				true => ContainerOrientation::Vertical,
				false => ContainerOrientation::Horizontal,
			},
			rel_size,
		}
	}

	// (rows, cols), corner is top left
	pub fn draw<W: Write>(&self, out: &mut W, mut cursor: (u16, u16), corner: (u16, u16), size: (u16, u16)) -> Result<(), std::io::Error> {
		self.write(out, &mut cursor, corner, size);
		out.flush()
	}

	fn write<W: Write>(&self, out: &mut W, cursor: &mut (u16, u16), origin: (u16, u16), size: (u16, u16)) {
		// Compute the sizes of each tile
		let total_size = match self.orientation {
			ContainerOrientation::Vertical => size.0,
			ContainerOrientation::Horizontal => size.1,
		};
		let total_rel_size = self.tiles.iter().map(|tile| match tile {
			TileType::Window(window) => window.config.rel_size,
			TileType::Container(container) => container.rel_size,
		}).sum::<f64>();
		let rel_unit_size = total_size as f64 / total_rel_size;
		// The height and width of each tile is computed based on the relative sizes
		let mut new_sizes: Vec<u16> = self.tiles.iter().map(|tile| match tile {
			TileType::Window(window) => (window.config.rel_size * rel_unit_size).floor() as u16,
			TileType::Container(container) => (container.rel_size * rel_unit_size).floor() as u16,
		}).collect();
		let remaining_space = total_size - new_sizes.iter().sum::<u16>();
		// The remainder is distributed starting with the first tile
		for i in 0..remaining_space {
			new_sizes[i as usize] += 1;
		}

		// Generate the new true sizes and origins
		let real_sizes = new_sizes.iter().map(|new_size| match self.orientation {
			ContainerOrientation::Vertical => (*new_size, size.1),
			ContainerOrientation::Horizontal => (size.0, *new_size),
		}).collect::<Vec<(u16, u16)>>();
		let real_origins = real_sizes.iter().scan(origin, |origin, size| {
			let updated_origin = *origin;
			*origin = match self.orientation {
				ContainerOrientation::Vertical => (origin.0 + size.0, origin.1),
				ContainerOrientation::Horizontal => (origin.0, origin.1 + size.1),
			};
			Some(updated_origin)
		}).collect::<Vec<(u16, u16)>>();

		// Call draw for each tile, replacing height or width as needed
		for (i, tile) in self.tiles.iter().enumerate() {
			match tile {
				TileType::Window(window) => window.write(out, cursor, real_origins[i], real_sizes[i]),
				TileType::Container(container) => container.write(out, cursor, real_origins[i], real_sizes[i]),
			}
		}
	}
}

fn offset(origin: (u16, u16), offset: (u16, u16)) -> (u16, u16) {
	(origin.0 + offset.0, origin.1 + offset.1)
}

pub struct Window<'a> {
	pub config: WindowConfig,
	pub display: &'a RefCell<dyn WindowDisplay + 'a>,
}
impl<'a> Window<'a> {
	pub fn new(display: &'a RefCell<dyn WindowDisplay + 'a>, config: WindowConfig) -> Window<'a> {
		Window {
			config,
			display,
		}
	}

	fn write<W: Write>(&self, out: &mut W, cursor: &mut (u16, u16), origin: (u16, u16), size: (u16, u16)) {
		let actions = self.display.borrow().render(size);
		for action in actions {
			match action {
				// NOTE: all crossterm functions use (col, row) as the format, but all of my
				// internal functions use (row, col) as the format. There will be some swapping
				// of the values here
				RenderAction::MoveTo(row, col) => {
					let real_pos = offset(origin, (row.min(size.0), col.min(size.1)));
					*cursor = real_pos;
					queue!(out, MoveTo(real_pos.1, real_pos.0)).unwrap();
				}
				RenderAction::MoveToNextLine(lines) => {
					let (cur_row, _) = *cursor;
					let new_row = cur_row + lines.min((size.0 + origin.0) - cur_row);
					*cursor = (new_row, origin.1);
					queue!(out, MoveTo(origin.1, new_row)).unwrap();
				}
				RenderAction::ClearToNextLine => {
					let (cur_row, cur_col) = *cursor;
					// Write ' ' until right bound of this window
					for _ in cur_col..(size.1 + origin.1) {
						queue!(out, Print(" ")).unwrap();
					}
					queue!(out, MoveTo(cur_col, cur_row)).unwrap();
				}
				RenderAction::ClearToEnd => {
					// Starting from cursor position, write ' ' until the end of the window
					// Each time the cursor reaches the end of the line, move to the next line
					let original_cursor = *cursor;
					for row in cursor.0..(size.0 + origin.0) {
						for _ in cursor.1..(size.1 + origin.1) {
							queue!(out, Print(" ")).unwrap();
						}
						queue!(out, MoveTo(origin.1, row+1)).unwrap();
					}
					// Return cursor to original position
					*cursor = original_cursor;
					queue!(out, MoveTo(original_cursor.1, original_cursor.0)).unwrap();
				}
				RenderAction::Write(pretty_string) => {
					let (_, cur_col) = *cursor;
					for style in pretty_string.style.iter() {
						match style {
							StyleProperty::FgColor(color) => queue!(out, SetForegroundColor(*color)).unwrap(),
							StyleProperty::BgColor(color) => queue!(out, SetBackgroundColor(*color)).unwrap(),
						}
					}
					if self.config.wrapping {
						// Print what could fit on the current line, then move to the next line and continue
						// printing what fits until the end of the string
						let mut to_write = pretty_string.contents.chars();
						while let Some(c) = to_write.next() {
							if cursor.1 >= size.1 + origin.1{
								// Move to the next line
								let new_row = cursor.0 + 1;
								// If the next line is out of bounds, stop writing
								if new_row >= size.0 + origin.0 {
									break;
								}
								*cursor = (new_row, origin.1);
								queue!(out, MoveTo(origin.1, new_row)).unwrap();
							}
							queue!(out, Print(format!("{}", c))).unwrap();
							cursor.1 += 1;
						}
					} else {
						let available_space = (size.1 + origin.1) - cur_col;
						// This does not appropriately handle characters that combine into one grapheme but
						// I don't care, I'm not writing a professional library here
						let to_write = pretty_string.contents.chars().take(available_space as usize).collect::<String>(); 
						queue!(out, Print(format!("{to_write}"))).unwrap();
						cursor.1 += to_write.len() as u16;
					}
					queue!(out, SetAttribute(Attribute::Reset)).unwrap();
				},
				RenderAction::HideCursor => queue!(out, Hide).unwrap(),
				RenderAction::ShowCursor => queue!(out, Show).unwrap(),
			}
		}
	}
}