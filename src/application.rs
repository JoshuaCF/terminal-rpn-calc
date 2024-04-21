use crate::command::{Command, Command::*, BinOp::*, UnOp::*};
use crate::tui_windows::*;

use crossterm::event::*;
use crossterm::style::Color;

use std::cell::RefCell;
use std::collections::HashMap;
use std::f64::consts::{E, PI};
use std::io::Error;

const STACK_SIZE: usize = 12;
struct NumStack {
    nums: [f64; STACK_SIZE],
}
impl NumStack {
    fn new() -> NumStack {
        NumStack { nums: [0.0; STACK_SIZE] }
    }
    fn rotate_in(&mut self, num: f64) {
        for i in (0..STACK_SIZE-1).rev() {
            self.nums[i+1] = self.nums[i];
        }
        self.nums[0] = num;
    }
    fn rotate_out(&mut self, num: f64) {
        for i in 1..STACK_SIZE {
            self.nums[i-1] = self.nums[i];
        }
        self.nums[0] = num;
    }

    fn add(&mut self) {
        let res = self.nums[1] + self.nums[0];
        self.rotate_out(res);
    }
    fn sub(&mut self) {
        let res = self.nums[1] - self.nums[0];
        self.rotate_out(res);
    }
    fn mul(&mut self) {
        let res = self.nums[1] * self.nums[0];
        self.rotate_out(res);
    }
    fn div(&mut self) {
        let res = self.nums[1] / self.nums[0];
        self.rotate_out(res);
    }
	fn intdiv(&mut self) {
		let result = (self.nums[1] / self.nums[0]) - (self.nums[1] % self.nums[0]) / self.nums[0];
		self.rotate_out(result);
	}
	fn r#mod(&mut self) {
		self.rotate_out(self.nums[1] % self.nums[0]);
	}

    fn neg(&mut self) {
        self.nums[0] = -self.nums[0];
    }

    fn swp(&mut self) {
        self.nums.swap(0, 1);
    }
	fn sqrt(&mut self) {
		self.nums[0] = self.nums[0].sqrt();
	}
	fn sqr(&mut self) {
		self.nums[0] = self.nums[0].powf(2.0);
	}
	fn nrt(&mut self) {
		let res = self.nums[1].powf(1.0 / self.nums[0]);
		self.rotate_out(res);
	}
	fn pow(&mut self) {
		let res = self.nums[1].powf(self.nums[0]);
		self.rotate_out(res);
	}
	fn exp(&mut self) {
		let res = self.nums[1] * (10.0f64).powf(self.nums[0]);
		self.rotate_out(res);
	}

	fn sin(&mut self) {
		self.nums[0] = self.nums[0].sin();
	}
	fn cos(&mut self) {
		self.nums[0] = self.nums[0].cos();
	}
	fn tan(&mut self) {
		self.nums[0] = self.nums[0].tan();
	}
	fn asin(&mut self) {
		self.nums[0] = self.nums[0].asin();
	}
	fn acos(&mut self) {
		self.nums[0] = self.nums[0].acos();
	}
	fn atan(&mut self) {
		self.nums[0] = self.nums[0].atan();
	}

	fn rad(&mut self) {
		self.nums[0] = (self.nums[0] / 360.0) * 2.0 * PI;
	}
	fn deg(&mut self) {
		self.nums[0] = (self.nums[0] * 360.0) / (2.0 * PI);
	}
}

pub struct Memory {
	mem: HashMap<char, f64>,
}
impl WindowDisplay for Memory {
	fn render(&self, _: (u16, u16)) -> Vec<RenderAction> {
		let mut actions = vec!();

		actions.push(RenderAction::MoveTo(0, 0));
		let mut entries = self.mem.iter().collect::<Vec<_>>();
		entries.sort_by(|a, b| a.0.cmp(b.0));
		for (k, v) in entries.iter() {
			actions.push(RenderAction::Write(PrettyString::new(format!("{}: ", k)).style(StyleProperty::FgColor(Color::Green))));
			let fmtd = if **v != 0.0 && (v.abs() < 1e-4 || v.abs() >= 1e11) {
				format!("{:.>20.8e}", v)
			} else {
				format!("{:.>20.8}", v)
			};
			actions.push(RenderAction::Write(PrettyString::new(format!("{fmtd}")).style(StyleProperty::FgColor(Color::Cyan))));
			actions.push(RenderAction::ClearToNextLine);
			actions.push(RenderAction::MoveToNextLine(1));
		}
		actions.push(RenderAction::ClearToEnd);

		actions
	}
}
impl Memory {
	pub fn new() -> Memory {
		Memory { mem: HashMap::new() }
	}
	fn store(&mut self, c: char, v: f64) {
		self.mem.insert(c, v);
	}
	fn recall(&self, c: char) -> Option<f64> {
		self.mem.get(&c).copied()
	}
	fn delete(&mut self, c: char) {
		self.mem.remove(&c);
	}
}

pub enum Response {
	NoAction,
	Exit,
}

pub struct Calculator<'a> {
    num_stack: NumStack,
	in_bfr: String,
	memory: &'a RefCell<Memory>,
}
impl WindowDisplay for Calculator<'_> {
	fn render(&self, _: (u16, u16)) -> Vec<RenderAction> {
		let mut actions = vec!();

		actions.push(RenderAction::MoveTo(0, 0));
		for v in self.num_stack.nums.iter().rev() {
			// TODO: These hardcoded boundaries are gross
			let fmtd = if *v != 0.0 && (v.abs() < 1e-4 || v.abs() >= 1e11) {
				format!("{:.>20.8e}", v)
			} else {
				format!("{:.>20.8}", v)
			};
			actions.push(RenderAction::Write(PrettyString::new(fmtd).style(StyleProperty::FgColor(Color::Cyan))));
			actions.push(RenderAction::ClearToNextLine);
			actions.push(RenderAction::MoveToNextLine(1));
		}
		actions.push(RenderAction::ClearToNextLine);
		actions.push(RenderAction::MoveToNextLine(1));
		actions.push(RenderAction::Write(PrettyString::new(self.in_bfr.clone()).style(StyleProperty::FgColor(Color::Magenta))));
		actions.push(RenderAction::ClearToEnd);

		actions
	}
}
impl<'a> Calculator<'a> {
	pub fn new(memory: &'a RefCell<Memory>) -> Calculator<'a> {
		Calculator { num_stack: NumStack::new(), in_bfr: String::with_capacity(256), memory }
	}
	pub fn process_event(&mut self, e: Event) -> Result<Response, Error> {
		let mut cmd = NoOp;
		if let Event::Key(ke) = e {
            if ke.kind != KeyEventKind::Press { return Ok(Response::NoAction); }
            cmd = match &ke.code {
                KeyCode::Esc => Exit,
                KeyCode::Backspace => RemoveFromBfr,
                KeyCode::Char(_) => self.process_char(ke)?,
                KeyCode::Enter => {
					if self.in_bfr.is_empty() {
						RotateIn(Some(self.num_stack.nums[0]))
					} else {
						self.process_text()?
					}
				},
                _ => NoOp,
            };
        }
		self.process_command(cmd)
	}

	pub fn process_command(&mut self, cmd: Command) -> Result<Response, Error> {
		match cmd {
            AppendToBfr(c) => self.in_bfr.push(c),
            BinOp(op) => {
                if !self.in_bfr.is_empty() {
					#[allow(clippy::single_match)]
                    match self.parse_num() {
                        Some(v) => self.num_stack.rotate_in(v),
                        None => (),
                    };
                    self.in_bfr.clear();
                }
                match op {
                    Add => self.num_stack.add(),
                    Sub => self.num_stack.sub(),
                    Mul => self.num_stack.mul(),
                    Div => self.num_stack.div(),
                    Swp => self.num_stack.swp(),
					Pow => self.num_stack.pow(),
					Rt => self.num_stack.nrt(),
					Exp => self.num_stack.exp(),
					IntDiv => self.num_stack.intdiv(),
					Mod => self.num_stack.r#mod(),
                }
                self.in_bfr.clear();
            },
            UnOp(op) => {
                match op {
                    Neg => self.num_stack.neg(),
					Sqr => self.num_stack.sqr(),
					Sqrt => self.num_stack.sqrt(),
					Sin => self.num_stack.sin(),
					Cos => self.num_stack.cos(),
					Tan => self.num_stack.tan(),
					Asin => self.num_stack.asin(),
					Acos => self.num_stack.acos(),
					Atan => self.num_stack.atan(),
					Rad => self.num_stack.rad(),
					Deg => self.num_stack.deg(),
					Pop => self.num_stack.rotate_out(self.num_stack.nums[1]),
                }
                self.in_bfr.clear();
            },
            RotateIn(v) => {
                self.num_stack.rotate_in(v.unwrap_or(self.num_stack.nums[0]));
                self.in_bfr.clear();
            }
            Exit => return Ok(Response::Exit),
            ClearBfr => self.in_bfr.clear(),
            RemoveFromBfr => {self.in_bfr.pop();},
			Sto(c) => {
				self.in_bfr.clear();
				let mut mem = self.memory.borrow_mut();
				mem.store(c, self.num_stack.nums[0]);
			},
			Rcl(c) => {
				self.in_bfr.clear();
				let mem = self.memory.borrow();
				if let Some(v) = mem.recall(c) {
					self.num_stack.rotate_in(v);
				}
			},
			Del(c) => {
				self.in_bfr.clear();
				let mut mem = self.memory.borrow_mut();
				mem.delete(c);
			},
            NoOp => (),
        }
		Ok(Response::NoAction)
	}

	fn process_char(&self, kchar: KeyEvent) -> Result<Command, std::io::Error> {
		let c = match kchar.code {
			KeyCode::Char(c) => c,
			_ => panic!(),
		};
		
		// TODO: Put these sorts of things into a configuration file
		if kchar.modifiers == KeyModifiers::SHIFT {
			match c.to_ascii_uppercase() {
				'+' => Ok(BinOp(Add)),
				'*' => Ok(BinOp(Mul)),
				'N' => Ok(UnOp(Neg)),
				'S' => Ok(BinOp(Swp)),
				'P' => Ok(BinOp(Pow)),
				'R' => Ok(BinOp(Rt)),
				'C' => Ok(UnOp(Pop)),
				'E' => Ok(BinOp(Exp)),
				'?' => Ok(BinOp(IntDiv)),
				'%' => Ok(BinOp(Mod)),
				ch => Ok(AppendToBfr(ch)),
			}
		} else if kchar.modifiers == KeyModifiers::NONE {
			match c.to_ascii_lowercase() {
				'-' => Ok(BinOp(Sub)),
				'/' => Ok(BinOp(Div)),
				ch => Ok(AppendToBfr(ch)),
			}
		} else if kchar.modifiers == KeyModifiers::CONTROL {
			match c.to_ascii_lowercase() {
				's' => {
					if self.in_bfr.len() != 1 { return Ok(NoOp); }
					let bfr_c = self.in_bfr.chars().next().unwrap();
					if bfr_c.is_ascii_alphabetic() {
						Ok(Sto(bfr_c))
					} else {
						Ok(NoOp)
					}
				},
				'd' => {
					if self.in_bfr.len() != 1 { return Ok(NoOp); }
					let bfr_c = self.in_bfr.chars().next().unwrap();
					if bfr_c.is_ascii_alphabetic() {
						Ok(Del(bfr_c))
					} else {
						Ok(NoOp)
					}
				},
				'r' => {
					if self.in_bfr.len() != 1 { return Ok(NoOp); }
					let bfr_c = self.in_bfr.chars().next().unwrap();
					if bfr_c.is_ascii_alphabetic() {
						Ok(Rcl(bfr_c))
					} else {
						Ok(NoOp)
					}
				},
				_ => Ok(NoOp),
			}
		} else {
			Ok(NoOp)
		}
	}

	fn process_text(&self) -> Result<Command, std::io::Error> {
		match self.in_bfr.as_str() {
			"sqrt" => Ok(UnOp(Sqrt)),
			"nrt" => Ok(BinOp(Rt)),
			"sqr" => Ok(UnOp(Sqr)),
			"pow" => Ok(BinOp(Pow)),
			"neg" => Ok(UnOp(Neg)),
			"swp" => Ok(BinOp(Swp)),
			"sin" => Ok(UnOp(Sin)),
			"cos" => Ok(UnOp(Cos)),
			"tan" => Ok(UnOp(Tan)),
			"asin" => Ok(UnOp(Asin)),
			"acos" => Ok(UnOp(Acos)),
			"atan" => Ok(UnOp(Atan)),
			"deg" => Ok(UnOp(Deg)),
			"rad" => Ok(UnOp(Rad)),
			_ => match self.parse_num() {
				Some(v) => Ok(RotateIn(Some(v))),
				None => Ok(ClearBfr),
			},
		}
	}

	fn parse_num(&self) -> Option<f64> {
		match self.in_bfr.as_str() {
			"pi" => Some(PI),
			"e" => Some(E),
			"g" => Some(9.80665),
			_ => {
				match self.in_bfr.parse::<f64>() {
					Ok(v) => Some(v),
					Err(_) => None,
				}
			}
		}
	}
}