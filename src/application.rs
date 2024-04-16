use crate::command::{Command, BinOp, UnOp};

use crossterm::queue;
use crossterm::event::*;
use crossterm::cursor::*;
use crossterm::terminal::*;

use std::f64::consts::PI;
use std::io::Error;
use std::io::Write;

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

    fn neg(&mut self) {
        self.nums[0] = -self.nums[0];
    }

    fn swp(&mut self) {
        let tmp = self.nums[0];
        self.nums[0] = self.nums[1];
        self.nums[1] = tmp;
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
    terminal: Box<dyn Write>,
    num_stack: NumStack,
	in_bfr: String,
}
impl Calculator {
	pub fn new(terminal: Box<dyn Write>) -> Calculator {
		Calculator { terminal, num_stack: NumStack::new(), in_bfr: String::with_capacity(256) }
	}
	pub fn process_event(&mut self, e: Event) -> Result<Response, Error> {
		let mut cmd = Command::NoOp;
		if let Event::Key(ke) = e {
            if ke.kind != KeyEventKind::Press { return Ok(Response::NoAction); }
            cmd = match &ke.code {
                KeyCode::Esc => Command::Exit,
                KeyCode::Backspace => Command::RemoveFromBfr,
                KeyCode::Char(_) => self.process_char(ke)?,
                KeyCode::Enter => self.process_text()?,
                _ => Command::NoOp,
            };
        }
		self.process_command(cmd)
	}

	pub fn process_command(&mut self, cmd: Command) -> Result<Response, Error> {
		match cmd {
			Command::Draw => {
				queue!(self.terminal, MoveTo(0, 0))?;
				for v in self.num_stack.nums.iter().rev() {
					// TODO: These hardcoded boundaries are gross
					let fmtd = if *v != 0.0 && (*v < 1e-4 || *v >= 1e11) {
						format!("{:.>20.8e}", v)
					} else {
						format!("{:.>20.8}", v)
					};
					write!(self.terminal, "{}", fmtd)?;
					queue!(self.terminal, Clear(ClearType::UntilNewLine), MoveToNextLine(1))?;
				}
				queue!(self.terminal, Clear(ClearType::UntilNewLine), MoveToNextLine(1))?;
				write!(self.terminal, "{}", self.in_bfr)?;
				queue!(self.terminal, Clear(ClearType::UntilNewLine))?;
				self.terminal.flush()?;
			},
            Command::AppendToBfr(c) => self.in_bfr.push(c),
            Command::BinOp(op) => {
                if !self.in_bfr.is_empty() {
                    match self.in_bfr.parse::<f64>() {
                        Ok(v) => self.num_stack.rotate_in(v),
                        Err(_) => (),
                    };
                    self.in_bfr.clear();
                }
                match op {
                    BinOp::Add => self.num_stack.add(),
                    BinOp::Sub => self.num_stack.sub(),
                    BinOp::Mul => self.num_stack.mul(),
                    BinOp::Div => self.num_stack.div(),
                    BinOp::Swp => self.num_stack.swp(),
					BinOp::Pow => self.num_stack.pow(),
					BinOp::Rt => self.num_stack.nrt(),
                }
                self.in_bfr.clear();
            },
            Command::UnOp(op) => {
                match op {
                    UnOp::Neg => self.num_stack.neg(),
					UnOp::Sqr => self.num_stack.sqr(),
					UnOp::Sqrt => self.num_stack.sqrt(),
					UnOp::Sin => self.num_stack.sin(),
					UnOp::Cos => self.num_stack.cos(),
					UnOp::Tan => self.num_stack.tan(),
					UnOp::Asin => self.num_stack.asin(),
					UnOp::Acos => self.num_stack.acos(),
					UnOp::Atan => self.num_stack.atan(),
					UnOp::Rad => self.num_stack.rad(),
					UnOp::Deg => self.num_stack.deg(),
					UnOp::Clr => self.num_stack.rotate_out(self.num_stack.nums[1]),
                }
                self.in_bfr.clear();
            },
            Command::RotateIn(v) => {
                self.num_stack.rotate_in(v.unwrap_or(self.num_stack.nums[0]));
                self.in_bfr.clear();
            }
            Command::Exit => return Ok(Response::Exit),
            Command::ClearBfr => self.in_bfr.clear(),
            Command::RemoveFromBfr => {self.in_bfr.pop();},
            Command::NoOp => (),
        }
		Ok(Response::NoAction)
	}

	fn process_char(&self, kchar: KeyEvent) -> Result<Command, std::io::Error> {
		if kchar.modifiers.bits() & !KeyModifiers::SHIFT.bits() != 0 { return Ok(Command::NoOp); }
		let c = match kchar.code {
			KeyCode::Char(c) => c,
			_ => panic!(),
		};
		
		// TODO: Put these sorts of things into a configuration file
		match c {
			'+' => Ok(Command::BinOp(BinOp::Add)),
			'-' => Ok(Command::BinOp(BinOp::Sub)),
			'*' => Ok(Command::BinOp(BinOp::Mul)),
			'/' => Ok(Command::BinOp(BinOp::Div)),
			'N' => Ok(Command::UnOp(UnOp::Neg)),
			'S' => Ok(Command::BinOp(BinOp::Swp)),
			'P' => Ok(Command::BinOp(BinOp::Pow)),
			'R' => Ok(Command::BinOp(BinOp::Rt)),
			'C' => Ok(Command::UnOp(UnOp::Clr)),
			ch => Ok(Command::AppendToBfr(ch)),
		}
	}

	fn process_text(&self) -> Result<Command, std::io::Error> {
		match self.in_bfr.as_str() {
			"sqrt" => Ok(Command::UnOp(UnOp::Sqrt)),
			"nrt" => Ok(Command::BinOp(BinOp::Rt)),
			"sqr" => Ok(Command::UnOp(UnOp::Sqr)),
			"pow" => Ok(Command::BinOp(BinOp::Pow)),
			"neg" => Ok(Command::UnOp(UnOp::Neg)),
			"swp" => Ok(Command::BinOp(BinOp::Swp)),
			"sin" => Ok(Command::UnOp(UnOp::Sin)),
			"cos" => Ok(Command::UnOp(UnOp::Cos)),
			"tan" => Ok(Command::UnOp(UnOp::Tan)),
			"asin" => Ok(Command::UnOp(UnOp::Asin)),
			"acos" => Ok(Command::UnOp(UnOp::Acos)),
			"atan" => Ok(Command::UnOp(UnOp::Atan)),
			"deg" => Ok(Command::UnOp(UnOp::Deg)),
			"rad" => Ok(Command::UnOp(UnOp::Rad)),
			"" => Ok(Command::RotateIn(None)),
			_ => {
				match self.in_bfr.parse::<f64>() {
					Ok(v) => Ok(Command::RotateIn(Some(v))),
					Err(_) => Ok(Command::ClearBfr),
				}
			}
		}
	}
}