use std::collections::HashMap;
use std::f64::consts::PI;

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
	pub nums: Vec<f64>,
	pub memory: HashMap<char, f64>,
}
impl Calculator {
	pub fn new(stack_size: usize) -> Calculator {
		Calculator {
			nums: vec!(0.0; stack_size),
			memory: HashMap::new(),
		}
	}

	pub fn process_command(&mut self, cmd: Command) {
		match cmd {
			// Stack commands
			Command::BinOp(op) => {
				match op {
					BinOp::Add => self.rotate_out(self.nums[1] + self.nums[0]),
					BinOp::Sub => self.rotate_out(self.nums[1] - self.nums[0]),
					BinOp::Mul => self.rotate_out(self.nums[1] * self.nums[0]),
					BinOp::Div => self.rotate_out(self.nums[1] / self.nums[0]),
					BinOp::IntDiv => self.rotate_out((self.nums[1] / self.nums[0]) % 1.0), // TODO: Test this!
					BinOp::Swp => {
						let tmp = self.nums[1];
						self.nums[1] = self.nums[0];
						self.nums[0] = tmp;
					},
					BinOp::Pow => self.rotate_out(self.nums[1].powf(self.nums[0])),
					BinOp::Root => self.rotate_out(self.nums[1].powf(1.0 / self.nums[0])),
					BinOp::Exp => self.rotate_out(self.nums[1] * (10.0f64).powf(self.nums[0])),
					BinOp::Mod => self.rotate_out(self.nums[1] % self.nums[0]),
				}
			},
			Command::UnOp(op) => {
				match op {
					UnOp::Neg => self.nums[0] = -self.nums[0],
					UnOp::Sqrt => self.nums[0] = self.nums[0].sqrt(),
					UnOp::Sqr => self.nums[0] = self.nums[0].powf(2.0),
					UnOp::Sin => self.nums[0] = self.nums[0].sin(),
					UnOp::Cos => self.nums[0] = self.nums[0].cos(),
					UnOp::Tan => self.nums[0] = self.nums[0].tan(),
					UnOp::Asin => self.nums[0] = self.nums[0].asin(),
					UnOp::Acos => self.nums[0] = self.nums[0].acos(),
					UnOp::Atan => self.nums[0] = self.nums[0].atan(),
					UnOp::Rad => self.nums[0] = (self.nums[0] / 360.0) * (2.0 * PI),
					UnOp::Deg => self.nums[0] = (self.nums[0] * 360.0) / (2.0 * PI),
					UnOp::Pop => self.rotate_out(self.nums[1]),
				}
			},
			Command::Push(val) => self.rotate_in(val.unwrap_or(self.nums[0])),
			// Memory commands
			Command::Sto(key) => { self.memory.insert(key, self.nums[0]); },
			Command::Del(key) => { self.memory.remove(&key); },
			Command::Rcl(key) => if let Some(v) = self.memory.get(&key).copied() { self.rotate_in(v); },
		}
	}

	fn rotate_in(&mut self, num: f64) {
		for i in (0..self.nums.len()-1).rev() {
			self.nums[i+1] = self.nums[i];
		}
		self.nums[0] = num;
	}
	fn rotate_out(&mut self, num: f64) {
		// Duplication of the first value on the stack is intentional
		// This is emulating the behavior of an RPN calculator I've used before
		for i in 1..self.nums.len() {
			self.nums[i-1] = self.nums[i];
		}
		self.nums[0] = num;
	}
}
