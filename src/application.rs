use crate::command::{Command, Command::*, BinOp::*, UnOp::*};
use crate::tui_windows::*;

use crossterm::event::*;

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

pub enum Response {
	NoAction,
	Exit,
}

pub struct Calculator {
    num_stack: NumStack,
	in_bfr: String,
}
impl WindowDisplay for Calculator {
	fn render(&self, _: (u16, u16)) -> Vec<RenderAction> {
		let mut actions = vec!();

		actions.push(RenderAction::HideCursor);
		actions.push(RenderAction::MoveTo(0, 0));
		for v in self.num_stack.nums.iter().rev() {
			// TODO: These hardcoded boundaries are gross
			let fmtd = if *v != 0.0 && (v.abs() < 1e-4 || v.abs() >= 1e11) {
				format!("{:.>20.8e}", v)
			} else {
				format!("{:.>20.8}", v)
			};
			actions.push(RenderAction::Write(PrettyString::new(fmtd)));
			actions.push(RenderAction::ClearToNextLine);
			actions.push(RenderAction::MoveToNextLine(1));
		}
		actions.push(RenderAction::ClearToNextLine);
		actions.push(RenderAction::MoveToNextLine(1));
		actions.push(RenderAction::Write(PrettyString::new(self.in_bfr.clone())));
		actions.push(RenderAction::ClearToEnd);
		actions.push(RenderAction::ShowCursor);

		actions
	}
}
impl Calculator {
	pub fn new() -> Calculator {
		Calculator { num_stack: NumStack::new(), in_bfr: String::with_capacity(256) }
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
            NoOp => (),
        }
		Ok(Response::NoAction)
	}

	fn process_char(&self, kchar: KeyEvent) -> Result<Command, std::io::Error> {
		if kchar.modifiers.bits() & !KeyModifiers::SHIFT.bits() != 0 { return Ok(NoOp); }
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
		} else {
			match c.to_ascii_lowercase() {
				'-' => Ok(BinOp(Sub)),
				'/' => Ok(BinOp(Div)),
				ch => Ok(AppendToBfr(ch)),
			}
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