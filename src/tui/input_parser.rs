use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::calculator::{BinOp, Command, UnOp};

#[derive(Clone, Copy)]
enum ParserCommand {
	Quit,
	DelChar,
	EvalBuf,

	CalcBinOp(BinOp),
	CalcUnOp(UnOp),
	CalcSto,
	CalcDel,
	CalcRcl,
}

// Used to change the behavior of pre-immediate buffer evaluation
#[derive(Clone, Copy)]
enum EvalMode {
	None, // Do nothing with the buffer
	Numbers, // If the buffer is a valid f64, push the number before executing, otherwise ignore
	Commands, // If the buffer is a valid string command, execute it before performing the immediate, otherwise ignore
	All, // Perform full buffer evaluation (Numbers + Commands)
}

// Used to change the behavior of evaluating an empty buffer
// TODO: This should be a calculator-level config
#[derive(Clone, Copy)]
enum EmptyEvalBehavior {
	None, // Perform no action
	PushZero, // Push a zero onto the stack
	PushLast, // Push the most recent stack value
}

pub struct ParserConfig {
	immediate_cmds: HashMap<KeyCode, ParserCommand>,
	// TODO: Make these forced to be parser commands, as string_cmds can never be incomplete
	string_cmds: HashMap<String, ParserCommand>,
	imm_eval_mode: EvalMode, // How should the buffer be handled when an immediate is executed?
	empty_eval_behavior: EmptyEvalBehavior,
}
impl Default for ParserConfig {
	fn default() -> Self {
		let mut config = ParserConfig {
			immediate_cmds: HashMap::new(),
			string_cmds: HashMap::new(),
			imm_eval_mode: EvalMode::Numbers,
			empty_eval_behavior: EmptyEvalBehavior::PushLast,
		};

		config.immediate_cmds.insert(KeyCode::Enter, ParserCommand::EvalBuf);
		config.immediate_cmds.insert(KeyCode::Backspace, ParserCommand::DelChar);
		config.immediate_cmds.insert(KeyCode::Delete, ParserCommand::DelChar);

		config.immediate_cmds.insert(KeyCode::Char('+'), ParserCommand::CalcBinOp(BinOp::Add));
		config.immediate_cmds.insert(KeyCode::Char('-'), ParserCommand::CalcBinOp(BinOp::Sub));
		config.immediate_cmds.insert(KeyCode::Char('*'), ParserCommand::CalcBinOp(BinOp::Mul));
		config.immediate_cmds.insert(KeyCode::Char('/'), ParserCommand::CalcBinOp(BinOp::Div));
		config.immediate_cmds.insert(KeyCode::Char('S'), ParserCommand::CalcBinOp(BinOp::Swp));
		config.immediate_cmds.insert(KeyCode::Char('P'), ParserCommand::CalcBinOp(BinOp::Pow));
		config.immediate_cmds.insert(KeyCode::Char('?'), ParserCommand::CalcBinOp(BinOp::IntDiv));
		config.immediate_cmds.insert(KeyCode::Char('%'), ParserCommand::CalcBinOp(BinOp::Mod));

		config.immediate_cmds.insert(KeyCode::Char('N'), ParserCommand::CalcUnOp(UnOp::Neg));
		config.immediate_cmds.insert(KeyCode::Char('C'), ParserCommand::CalcUnOp(UnOp::Pop));

		config.immediate_cmds.insert(KeyCode::Char('F'), ParserCommand::CalcSto);
		config.immediate_cmds.insert(KeyCode::Char('D'), ParserCommand::CalcDel);
		config.immediate_cmds.insert(KeyCode::Char('R'), ParserCommand::CalcRcl);

		config.string_cmds.insert("quit".into(), ParserCommand::Quit);

		config.string_cmds.insert("add".into(), ParserCommand::CalcBinOp(BinOp::Add));
		config.string_cmds.insert("sub".into(), ParserCommand::CalcBinOp(BinOp::Sub));
		config.string_cmds.insert("mul".into(), ParserCommand::CalcBinOp(BinOp::Mul));
		config.string_cmds.insert("div".into(), ParserCommand::CalcBinOp(BinOp::Div));
		config.string_cmds.insert("swp".into(), ParserCommand::CalcBinOp(BinOp::Swp));
		config.string_cmds.insert("pow".into(), ParserCommand::CalcBinOp(BinOp::Pow));
		config.string_cmds.insert("root".into(), ParserCommand::CalcBinOp(BinOp::Root));
		config.string_cmds.insert("exp".into(), ParserCommand::CalcBinOp(BinOp::Exp));
		config.string_cmds.insert("intdiv".into(), ParserCommand::CalcBinOp(BinOp::IntDiv));
		config.string_cmds.insert("mod".into(), ParserCommand::CalcBinOp(BinOp::Mod));

		config.string_cmds.insert("neg".into(), ParserCommand::CalcUnOp(UnOp::Neg));
		config.string_cmds.insert("sqrt".into(), ParserCommand::CalcUnOp(UnOp::Sqrt));
		config.string_cmds.insert("sqr".into(), ParserCommand::CalcUnOp(UnOp::Sqr));
		config.string_cmds.insert("sin".into(), ParserCommand::CalcUnOp(UnOp::Sin));
		config.string_cmds.insert("cos".into(), ParserCommand::CalcUnOp(UnOp::Cos));
		config.string_cmds.insert("tan".into(), ParserCommand::CalcUnOp(UnOp::Tan));
		config.string_cmds.insert("asin".into(), ParserCommand::CalcUnOp(UnOp::Asin));
		config.string_cmds.insert("acos".into(), ParserCommand::CalcUnOp(UnOp::Acos));
		config.string_cmds.insert("atan".into(), ParserCommand::CalcUnOp(UnOp::Atan));
		config.string_cmds.insert("rad".into(), ParserCommand::CalcUnOp(UnOp::Rad));
		config.string_cmds.insert("deg".into(), ParserCommand::CalcUnOp(UnOp::Deg));
		config.string_cmds.insert("pop".into(), ParserCommand::CalcUnOp(UnOp::Pop));

		config
	}
}

pub enum ExternalCommand {
	Quit,
	CalcCmd(Command),
}

pub struct Parser {
	pub bfr: Vec<char>,
	pub config: ParserConfig,
}
impl Parser {
	pub fn new() -> Parser {
		Parser {
			bfr: Vec::new(),
			config: ParserConfig::default(),
		}
	}

	// When pressing a button with an immediate action, the current buffer is evaluated to
	// determine if an action should occur before the immediate action, which can cause
	// more than once action to be performed in a single press, hence the need for `Vec`

	// Maybe return an `Option` rather than an empty `Vec`?
	pub fn parse(&mut self, ke: KeyEvent) -> Vec<ExternalCommand> {
		let mut actions = Vec::new();

		// I don't know a better way to return early on non-presses
		match ke.kind {
			KeyEventKind::Press => (),
			_ => return actions
		}

		if let Some(inc_cmd) = self.config.immediate_cmds.get(&ke.code) {
			// Special cases, no pre-buffer evaluation should be done
			match *inc_cmd {
				ParserCommand::DelChar => {
					self.bfr.pop();
					return actions;
				},
				ParserCommand::EvalBuf => {
					if let Some(cmd) = self.eval_buffer(false) {
						actions.push(cmd);
						self.bfr.clear();
					}
					return actions;
				},
				_ => { // If not a special case, do pre-eval
					if let Some(cmd) = self.eval_buffer(true) {
						actions.push(cmd);
						self.bfr.clear();
					}
				},
			}

			let register = if self.bfr.len() == 1 {
				Some(self.bfr[0])
			} else {
				None
			};

			match *inc_cmd {
				ParserCommand::Quit => actions.push(ExternalCommand::Quit),

				ParserCommand::CalcBinOp(op) => actions.push(ExternalCommand::CalcCmd(Command::BinOp(op))),
				ParserCommand::CalcUnOp(op) => actions.push(ExternalCommand::CalcCmd(Command::UnOp(op))),
				ParserCommand::CalcSto => if let Some(c) = register {
					self.bfr.clear();
					actions.push(ExternalCommand::CalcCmd(Command::Sto(c)));
				},
				ParserCommand::CalcDel => if let Some(c) = register {
					self.bfr.clear();
					actions.push(ExternalCommand::CalcCmd(Command::Del(c)));
				},
				ParserCommand::CalcRcl => if let Some(c) = register {
					self.bfr.clear();
					actions.push(ExternalCommand::CalcCmd(Command::Rcl(c)));
				},

				ParserCommand::DelChar => unreachable!(),
				ParserCommand::EvalBuf => unreachable!(),
			}

			self.bfr.clear();
		} else if let KeyCode::Char(c) = ke.code {
			self.bfr.push(c);
		}

		actions
	}

	fn eval_buffer(&self, pre_eval: bool) -> Option<ExternalCommand> {
		if !pre_eval && self.bfr.len() == 0 {
			match self.config.empty_eval_behavior {
				EmptyEvalBehavior::None => (),
				EmptyEvalBehavior::PushZero => return Some(ExternalCommand::CalcCmd(Command::Push(Some(0.0f64)))),
				EmptyEvalBehavior::PushLast => return Some(ExternalCommand::CalcCmd(Command::Push(None))),
			}
		}

		let bfr_string = self.bfr.iter().collect::<String>();
		let num_parse = bfr_string.parse::<f64>();

		let mode = match pre_eval {
			true => self.config.imm_eval_mode,
			false => EvalMode::All,
		};

		// TODO: Reduce duplication here? Might not be worth it in such a small case
		match mode {
			EvalMode::None => return None,
			EvalMode::Numbers => {
				if let Ok(v) = num_parse {
					return Some(ExternalCommand::CalcCmd(Command::Push(Some(v))));
				} else {
					return None;
				}
			},
			EvalMode::Commands => {
				if let Some(inc_cmd) = self.config.string_cmds.get(bfr_string.as_str()) {
					match *inc_cmd {
						ParserCommand::Quit => return Some(ExternalCommand::Quit),

						ParserCommand::CalcBinOp(op) => return Some(ExternalCommand::CalcCmd(Command::BinOp(op))),
						ParserCommand::CalcUnOp(op) => return Some(ExternalCommand::CalcCmd(Command::UnOp(op))),

						_ => panic!("Invalid command in eval_buffer"),
					}
				} else {
					return None;
				}
			},
			EvalMode::All => {
				if let Ok(v) = num_parse {
					return Some(ExternalCommand::CalcCmd(Command::Push(Some(v))));
				} else if let Some(inc_cmd) = self.config.string_cmds.get(bfr_string.as_str()) {
					match *inc_cmd {
						ParserCommand::Quit => return Some(ExternalCommand::Quit),

						ParserCommand::CalcBinOp(op) => return Some(ExternalCommand::CalcCmd(Command::BinOp(op))),
						ParserCommand::CalcUnOp(op) => return Some(ExternalCommand::CalcCmd(Command::UnOp(op))),

						_ => panic!("Invalid command in eval_buffer"),
					}
				} else {
					return None;
				}
			},
		}
	}
}
