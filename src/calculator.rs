use std::collections::HashMap;
use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

// Configuration
// What should the calculator do when receiving a `Push` with no value?
#[derive(Debug, Serialize, Deserialize)]
enum EmptyPushBehavior {
	None, // Ignore it
	Zero, // Push zero
	Last, // Push the most recent value
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CalculatorConfig {
	empty_push_behavior: EmptyPushBehavior,
	stack_size: usize,
}
impl Default for CalculatorConfig {
	fn default() -> Self {
		Self {
			stack_size: 8,
			empty_push_behavior: EmptyPushBehavior::Last,
		}
	}
}

// Actions
#[derive(Clone, Copy, Debug)]
pub enum Command {
	BinOp(BinOp),
	UnOp(UnOp),
	Push(Option<f64>),
	Sto(char),
	Del(char),
	Rcl(char),
}
#[derive(Clone, Copy, Debug)]
pub enum BinOp {
	Add,
	Sub,
	Mul,
	Div,
	Swp,
	Pow,
	Root,
	Exp, // 10^x
	IntDiv,
	Mod,
}
#[derive(Clone, Copy, Debug)]
pub enum UnOp {
	Neg,
	Sqrt,
	Sqr,
	Sin,
	Cos,
	Tan,
	Asin,
	Acos,
	Atan,
	Rad,
	Deg,
	Pop,
}

#[derive(Debug)]
pub struct Calculator {
	pub stack: Vec<f64>,
	// TODO: there's really no reason this has to be 'char', it could be strings
	pub memory: HashMap<char, f64>,
	pub config: CalculatorConfig,
}
impl Calculator {
	pub fn new(config: CalculatorConfig) -> Calculator {
		Calculator {
			stack: vec![0.0; config.stack_size],
			memory: HashMap::new(),
			config,
		}
	}

	pub fn process_command(&mut self, cmd: Command) {
		match cmd {
			// Stack commands
			Command::BinOp(op) => {
				match op {
					BinOp::Add => self.rotate_out_and_set_last(self.stack[1] + self.stack[0]),
					BinOp::Sub => self.rotate_out_and_set_last(self.stack[1] - self.stack[0]),
					BinOp::Mul => self.rotate_out_and_set_last(self.stack[1] * self.stack[0]),
					BinOp::Div => self.rotate_out_and_set_last(self.stack[1] / self.stack[0]),
					BinOp::IntDiv => self.rotate_out_and_set_last((self.stack[1] / self.stack[0]) % 1.0), // TODO: Test this!
					BinOp::Swp => {
						let tmp = self.stack[1];
						self.stack[1] = self.stack[0];
						self.stack[0] = tmp;
					},
					BinOp::Pow => self.rotate_out_and_set_last(self.stack[1].powf(self.stack[0])),
					BinOp::Root => self.rotate_out_and_set_last(self.stack[1].powf(1.0 / self.stack[0])),
					BinOp::Exp => self.rotate_out_and_set_last(self.stack[1] * (10.0f64).powf(self.stack[0])),
					BinOp::Mod => self.rotate_out_and_set_last(self.stack[1] % self.stack[0]),
				}
			},
			Command::UnOp(op) => match op {
				UnOp::Neg => self.stack[0] = -self.stack[0],
				UnOp::Sqrt => self.stack[0] = self.stack[0].sqrt(),
				UnOp::Sqr => self.stack[0] = self.stack[0].powf(2.0),
				UnOp::Sin => self.stack[0] = self.stack[0].sin(),
				UnOp::Cos => self.stack[0] = self.stack[0].cos(),
				UnOp::Tan => self.stack[0] = self.stack[0].tan(),
				UnOp::Asin => self.stack[0] = self.stack[0].asin(),
				UnOp::Acos => self.stack[0] = self.stack[0].acos(),
				UnOp::Atan => self.stack[0] = self.stack[0].atan(),
				UnOp::Rad => self.stack[0] = (self.stack[0] / 360.0) * (2.0 * PI),
				UnOp::Deg => self.stack[0] = (self.stack[0] * 360.0) / (2.0 * PI),
				UnOp::Pop => self.rotate_out_and_set_last(self.stack[1]),
			},
			Command::Push(val) => match val {
				Some(v) => self.rotate_in(v),
				None => match self.config.empty_push_behavior {
					EmptyPushBehavior::None => (),
					EmptyPushBehavior::Zero => self.rotate_in(0.0f64),
					EmptyPushBehavior::Last => self.rotate_in(self.stack[0]),
				},
			},
			// Memory commands
			Command::Sto(key) => {
				self.memory.insert(key, self.stack[0]);
			},
			Command::Del(key) => {
				self.memory.remove(&key);
			},
			Command::Rcl(key) => {
				if let Some(v) = self.memory.get(&key).copied() {
					self.rotate_in(v);
				}
			},
		}
	}

	fn rotate_in(&mut self, num: f64) {
		for i in (0..self.stack.len() - 1).rev() {
			self.stack[i + 1] = self.stack[i];
		}
		self.stack[0] = num;
	}
	fn rotate_out_and_set_last(&mut self, num: f64) {
		// Duplication of the first value on the stack is intentional
		// This is emulating the behavior of an RPN calculator I've used before
		// TODO: Make duplication configurable
		for i in 1..self.stack.len() {
			self.stack[i - 1] = self.stack[i];
		}
		self.stack[0] = num;
	}
}
