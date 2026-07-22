use std::collections::HashMap;
use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

// Configuration
/// Determines the calculator's behavior when receiving a push command with no content.
#[derive(Default, Debug, Serialize, Deserialize)]
enum EmptyPushBehavior {
	/// Ignore it
	None,
	/// Push zero
	Zero,
	/// Push the most recent value
	#[default]
	Last,
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
			empty_push_behavior: EmptyPushBehavior::default(),
		}
	}
}

// Actions
/// Top-level grouping of actions the calculator can take.
#[derive(Clone, Debug)]
pub enum Command {
	/// Perform an operation involving the bottom two values of the stack
	BinOp(BinOp),
	/// Perform an operation involving the bottom value of the stack
	UnOp(UnOp),
	/// Push a value onto the stack. Behavior in the case of `None` determined by config
	Push(Option<f64>),
	/// Store a value into memory
	Sto(String),
	/// Delete a value from memory
	Del(String),
	/// Push a value from memory onto the stack
	Rcl(String),
}
/// Operations involving the bottom two values on the stack.
///
/// All operations here (except `BinOp::Swp`) will remove the bottom two values from the stack and push the result of the
/// operation onto the stack.
#[derive(Clone, Copy, Debug)]
pub enum BinOp {
	/// Add the bottom two values together
	Add,
	/// Subtract the bottom value from the second-bottom value
	Sub,
	/// Multiply the bottom two values together
	Mul,
	/// Divide the second-bottom value on the stack by the bottom value on the stack
	Div,
	/// Swap the bottom two values of the stack
	Swp,
	/// Raise the bottom value of the stack to the power of the second-bottom value
	Pow,
	/// Raise the bottom value of the stack to the power of the one over the second-bottom value
	Root,
	/// Raise 10 to the power of the bottom value of the stack, then multiply that with the
	/// second-bottom value
	Exp, // 10^x
	/// Like `BinOp::Div`, but uses euclidean division instead
	IntDiv,
	/// Computes the second-bottom value of the stack modulo the bottom value of the stack
	Mod,
}
/// Operations involving only the bottom value of the stack.
///
/// All trigonometric functions assume the input is in radians.
#[derive(Clone, Copy, Debug)]
pub enum UnOp {
	/// Multiply the bottom value by -1
	Neg,
	/// Compute the square root of the bottom value
	Sqrt,
	/// Square the bottom value
	Sqr,
	/// Compute the sine of the bottom value
	Sin,
	/// Compute the cosine of the bottom value
	Cos,
	/// Compute the tangent of the bottom value
	Tan,
	/// Compute the arcsin of the bottom value
	Asin,
	/// Compute the arccosine of the bottom value
	Acos,
	/// Compute the arctangent of the bottom value
	Atan,
	/// Convert the bottom value from degrees to radians
	Rad,
	/// Convert the bottom value from radians to degrees
	Deg,
	/// Remove the bottom value of the stack
	Pop,
}

#[derive(Debug)]
pub struct Calculator {
	pub stack: Vec<f64>,
	pub memory: HashMap<String, f64>,
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
					BinOp::IntDiv => {
						self.rotate_out_and_set_last(self.stack[1].div_euclid(self.stack[0]))
					},
					BinOp::Swp => {
						let tmp = self.stack[1];
						self.stack[1] = self.stack[0];
						self.stack[0] = tmp;
					},
					BinOp::Pow => self.rotate_out_and_set_last(self.stack[1].powf(self.stack[0])),
					BinOp::Root => {
						self.rotate_out_and_set_last(self.stack[1].powf(1.0 / self.stack[0]))
					},
					BinOp::Exp => {
						self.rotate_out_and_set_last(self.stack[1] * (10.0f64).powf(self.stack[0]))
					},
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
