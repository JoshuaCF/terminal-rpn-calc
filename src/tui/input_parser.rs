use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use serde::{Deserialize, Serialize, Serializer, Deserializer};
use serde::ser::Error as SerError;
use serde::ser::{SerializeMap};
use serde::de::Error as DeError;
use serde::de::{VariantAccess, Unexpected, Visitor, MapAccess};

use crate::calculator::{BinOp, Command, UnOp};

/// Used to change the behavior of pre-immediate buffer evaluation.
#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub enum EvalMode {
	/// Do nothing with the buffer
    None,
	/// If the buffer is a valid f64, push the number before executing, otherwise ignore
	#[default]
    Numbers,
	/// If the buffer is a valid string command, execute it before performing the immediate, otherwise ignore
    Commands,
	/// Perform full buffer evaluation (Numbers + Commands)
    All,
}

// Configuration
/// Keybinds which execute an operation as soon as they're detected, as contrasted with entering the
/// buffer.
///
/// This is just a wrapper for `HashMap<KeyCode, ParserCommand>` which is needed to implement custom
/// serialization and deserialization due to the [toml] crate not supporting hashmaps with
/// non-string keys.
pub struct ImmediateCmdConfig(HashMap<KeyCode, ParserCommand>);
impl From<HashMap<KeyCode, ParserCommand>> for ImmediateCmdConfig {
	fn from(value: HashMap<KeyCode, ParserCommand>) -> Self {
	    Self(value)
	}
}
impl From<ImmediateCmdConfig> for HashMap<KeyCode, ParserCommand> {
	fn from(value: ImmediateCmdConfig) -> Self {
	    value.0
	}
}
impl Serialize for ImmediateCmdConfig {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
	    S: serde::Serializer
	{
		let mut map_serializer = serializer.serialize_map(None)?;

		for (key, value) in self.0.iter() {
			match key {
				KeyCode::Char(c) => map_serializer.serialize_entry(&c.to_string(), value)?,
				KeyCode::Enter => map_serializer.serialize_entry("enter", value)?,
				KeyCode::Backspace => map_serializer.serialize_entry("backspace", value)?,
				KeyCode::Delete => map_serializer.serialize_entry("delete", value)?,
				_ => return Err(S::Error::custom("unable to convert keycode to string")),
			}
		}

		map_serializer.end()
	}
}
impl<'de> Deserialize<'de> for ImmediateCmdConfig {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
	    D: serde::Deserializer<'de>
	{
		struct ImmediateCmdConfigVisitor;
		impl<'de> Visitor<'de> for ImmediateCmdConfigVisitor {
			type Value = ImmediateCmdConfig;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
			    write!(formatter, "a map of characters or key names to actions")
			}

			fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
			where
			    A: serde::de::MapAccess<'de>,
			{
			    let mut map = HashMap::new();

				while let Some((key_string, value)) = access.next_entry::<String, ParserCommand>()? {
					let key = match key_string.as_str() {
						"enter" => KeyCode::Enter,
						"backspace" => KeyCode::Backspace,
						"delete" => KeyCode::Delete,
						maybe_char => {
							if maybe_char.chars().count() == 1 {
								KeyCode::Char(maybe_char.chars().next().unwrap())
							} else {
								return Err(A::Error::invalid_value(Unexpected::Str(maybe_char), &"expected a single character or a key name"));
							}
						},
					};
					map.insert(key, value);
				}

				Ok(ImmediateCmdConfig(map))
			}
		}

		deserializer.deserialize_map(ImmediateCmdConfigVisitor)
	}
}

/// Configuration for keybinds and commands.
#[derive(Serialize, Deserialize)]
pub struct ParserConfig {
	/// Commands that execute upon a single keypress
    pub immediate_cmds: ImmediateCmdConfig,
	/// Commands that execute when evaluating a typed string
    pub string_cmds: HashMap<String, ParserCommand>,
	/// Determines what is done with the buffer when executing an immediate
    pub imm_eval_mode: EvalMode,
}
impl Default for ParserConfig {
    fn default() -> Self {
		let mut immediate_cmds = HashMap::new();
		let mut string_cmds = HashMap::new();
		let imm_eval_mode = EvalMode::default();

		// TODO: literally all of this needs to be configurable
		immediate_cmds
            .insert(KeyCode::Enter, ParserCommand::EvalBuf);
        immediate_cmds
            .insert(KeyCode::Backspace, ParserCommand::DelChar);
        immediate_cmds
            .insert(KeyCode::Delete, ParserCommand::DelChar);

        immediate_cmds
            .insert(KeyCode::Char('+'), ParserCommand::CalcBinOp(BinOp::Add));
        immediate_cmds
            .insert(KeyCode::Char('-'), ParserCommand::CalcBinOp(BinOp::Sub));
        immediate_cmds
            .insert(KeyCode::Char('*'), ParserCommand::CalcBinOp(BinOp::Mul));
        immediate_cmds
            .insert(KeyCode::Char('/'), ParserCommand::CalcBinOp(BinOp::Div));
        immediate_cmds
            .insert(KeyCode::Char('S'), ParserCommand::CalcBinOp(BinOp::Swp));
        immediate_cmds
            .insert(KeyCode::Char('P'), ParserCommand::CalcBinOp(BinOp::Pow));
        immediate_cmds
            .insert(KeyCode::Char('?'), ParserCommand::CalcBinOp(BinOp::IntDiv));
        immediate_cmds
            .insert(KeyCode::Char('%'), ParserCommand::CalcBinOp(BinOp::Mod));

        immediate_cmds
            .insert(KeyCode::Char('N'), ParserCommand::CalcUnOp(UnOp::Neg));
        immediate_cmds
            .insert(KeyCode::Char('C'), ParserCommand::CalcUnOp(UnOp::Pop));

        immediate_cmds
            .insert(KeyCode::Char('F'), ParserCommand::CalcStore);
        immediate_cmds
            .insert(KeyCode::Char('D'), ParserCommand::CalcDelete);
        immediate_cmds
            .insert(KeyCode::Char('R'), ParserCommand::CalcRecall);

        string_cmds
            .insert("quit".into(), ParserCommand::Quit);

        string_cmds
            .insert("add".into(), ParserCommand::CalcBinOp(BinOp::Add));
        string_cmds
            .insert("sub".into(), ParserCommand::CalcBinOp(BinOp::Sub));
        string_cmds
            .insert("mul".into(), ParserCommand::CalcBinOp(BinOp::Mul));
        string_cmds
            .insert("div".into(), ParserCommand::CalcBinOp(BinOp::Div));
        string_cmds
            .insert("swp".into(), ParserCommand::CalcBinOp(BinOp::Swp));
        string_cmds
            .insert("pow".into(), ParserCommand::CalcBinOp(BinOp::Pow));
        string_cmds
            .insert("root".into(), ParserCommand::CalcBinOp(BinOp::Root));
        string_cmds
            .insert("exp".into(), ParserCommand::CalcBinOp(BinOp::Exp));
        string_cmds
            .insert("intdiv".into(), ParserCommand::CalcBinOp(BinOp::IntDiv));
        string_cmds
            .insert("mod".into(), ParserCommand::CalcBinOp(BinOp::Mod));

        string_cmds
            .insert("neg".into(), ParserCommand::CalcUnOp(UnOp::Neg));
        string_cmds
            .insert("sqrt".into(), ParserCommand::CalcUnOp(UnOp::Sqrt));
        string_cmds
            .insert("sqr".into(), ParserCommand::CalcUnOp(UnOp::Sqr));
        string_cmds
            .insert("sin".into(), ParserCommand::CalcUnOp(UnOp::Sin));
        string_cmds
            .insert("cos".into(), ParserCommand::CalcUnOp(UnOp::Cos));
        string_cmds
            .insert("tan".into(), ParserCommand::CalcUnOp(UnOp::Tan));
        string_cmds
            .insert("asin".into(), ParserCommand::CalcUnOp(UnOp::Asin));
        string_cmds
            .insert("acos".into(), ParserCommand::CalcUnOp(UnOp::Acos));
        string_cmds
            .insert("atan".into(), ParserCommand::CalcUnOp(UnOp::Atan));
        string_cmds
            .insert("rad".into(), ParserCommand::CalcUnOp(UnOp::Rad));
        string_cmds
            .insert("deg".into(), ParserCommand::CalcUnOp(UnOp::Deg));
        string_cmds
            .insert("pop".into(), ParserCommand::CalcUnOp(UnOp::Pop));

        Self {
			immediate_cmds: immediate_cmds.into(),
			string_cmds,
			imm_eval_mode
		}
    }
}

// Actions
#[derive(Clone, Copy, Debug)]
pub enum ParserCommand {
    Quit,
    DelChar,
    EvalBuf,

    CalcBinOp(BinOp),
    CalcUnOp(UnOp),
    CalcStore,
    CalcDelete,
    CalcRecall,
}
impl ParserCommand {
	/// Name of the enum for (de)serialization
	fn enum_name() -> &'static str {
		"ParserCommand"
	}
}
// Deriving serialize/deserialize maps the data structure in a dissatisfactory manner
impl Serialize for ParserCommand {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
	    S: Serializer
	{
		let variant: ParserCommandVariant = (*self).into();
		serializer.serialize_unit_variant(Self::enum_name(), variant.index(), variant.name())
	}
}
impl<'de> Deserialize<'de> for ParserCommand {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
	    D: Deserializer<'de>
	{
		// Most of the functions here can just forward to the ParserCommandVariantVisitor
		// TODO: Maybe DeserializeSeed might be better then?
		struct ParserCommandVisitor;
		impl<'de> Visitor<'de> for ParserCommandVisitor {
			type Value = ParserCommand;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				write!(formatter, "a valid parser command")
			}

			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
			where
			    E: serde::de::Error,
			{
				if let Ok(v) = ParserCommand::try_from(ParserCommandVariantVisitor.visit_str::<E>(v)?) {
					Ok(v)
				} else {
					Err(E::invalid_value(Unexpected::Str(v), &"a string identifying a unit variant"))
				}
			}

			fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
			where
			    E: serde::de::Error,
			{
				if let Ok(v) = ParserCommand::try_from(ParserCommandVariantVisitor.visit_i64::<E>(v)?) {
					Ok(v)
				} else {
					Err(E::invalid_value(Unexpected::Signed(v), &"an integer identifying a unit variant"))
				}
			}

			fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
			where
			    E: serde::de::Error,
			{
				if let Ok(v) = ParserCommand::try_from(ParserCommandVariantVisitor.visit_i128::<E>(v)?) {
					Ok(v)
				} else {
					Err(E::invalid_value(Unexpected::Signed(v as i64), &"an integer identifying a unit variant"))
				}
			}

			fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
			where
			    E: serde::de::Error,
			{
				if let Ok(v) = ParserCommand::try_from(ParserCommandVariantVisitor.visit_u32::<E>(v)?) {
					Ok(v)
				} else {
					Err(E::invalid_value(Unexpected::Unsigned(v as u64), &"an integer identifying a unit variant"))
				}
			}

			fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
			where
			    E: serde::de::Error,
			{
				if let Ok(v) = ParserCommand::try_from(ParserCommandVariantVisitor.visit_u64::<E>(v)?) {
					Ok(v)
				} else {
					Err(E::invalid_value(Unexpected::Unsigned(v), &"an integer identifying a unit variant"))
				}
			}

			fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
			where
			    E: serde::de::Error,
			{
				if let Ok(v) = ParserCommand::try_from(ParserCommandVariantVisitor.visit_u128::<E>(v)?) {
					Ok(v)
				} else {
					Err(E::invalid_value(Unexpected::Unsigned(v as u64), &"an integer identifying a unit variant"))
				}
			}

			fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
			where
				A: serde::de::EnumAccess<'de>,
			{
				let (result, accessor) = data.variant::<ParserCommandVariant>()?;

				if let Ok(v) = ParserCommand::try_from(result) {
					accessor.unit_variant()?;
					Ok(v)
				} else {
					unimplemented!("only unit variants at time of writing")
				}
			}
		}

		deserializer.deserialize_enum("ParserCommand", ParserCommandVariant::ALL_NAMES, ParserCommandVisitor)
	}
}

#[derive(Clone, Copy, Debug)]
pub enum ParserCommandVariant {
	Quit,
	DelChar,
	EvalBuf,
	Add,
	Sub,
	Mul,
	Div,
	Swp,
	Pow,
	Root,
	Exp,
	IntDiv,
	Mod,
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
	CalcStore,
	CalcDelete,
	CalcRecall,
}
impl ParserCommandVariant {
	const ALL_NAMES: &'static [&'static str] = &[
		Self::QUIT,
		Self::DEL_CHAR,
		Self::EVAL_BUF,
		Self::ADD,
		Self::SUB,
		Self::MUL,
		Self::DIV,
		Self::SWP,
		Self::POW,
		Self::ROOT,
		Self::EXP,
		Self::INT_DIV,
		Self::MOD,
		Self::NEG,
		Self::SQRT,
		Self::SQR,
		Self::SIN,
		Self::COS,
		Self::TAN,
		Self::ASIN,
		Self::ACOS,
		Self::ATAN,
		Self::RAD,
		Self::DEG,
		Self::POP,
		Self::CALC_STORE,
		Self::CALC_DELETE,
		Self::CALC_RECALL,
	];

	const QUIT: &'static str = "Quit";
	const DEL_CHAR: &'static str = "DelChar";
	const EVAL_BUF: &'static str = "EvalBuf";
	const ADD: &'static str = "Add";
	const SUB: &'static str = "Sub";
	const MUL: &'static str = "Mul";
	const DIV: &'static str = "Div";
	const SWP: &'static str = "Swp";
	const POW: &'static str = "Pow";
	const ROOT: &'static str = "Root";
	const EXP: &'static str = "Exp";
	const INT_DIV: &'static str = "IntDiv";
	const MOD: &'static str = "Mod";
	const NEG: &'static str = "Neg";
	const SQRT: &'static str = "Sqrt";
	const SQR: &'static str = "Sqr";
	const SIN: &'static str = "Sin";
	const COS: &'static str = "Cos";
	const TAN: &'static str = "Tan";
	const ASIN: &'static str = "Asin";
	const ACOS: &'static str = "Acos";
	const ATAN: &'static str = "Atan";
	const RAD: &'static str = "Rad";
	const DEG: &'static str = "Deg";
	const POP: &'static str = "Pop";
	const CALC_STORE: &'static str = "CalcStore";
	const CALC_DELETE: &'static str = "CalcDelete";
	const CALC_RECALL: &'static str = "CalcRecall";

	/// Name of a variant for (de)serialization
	const fn name(self) -> &'static str {
		match self {
			Self::Quit => Self::QUIT,
			Self::DelChar => Self::DEL_CHAR,
			Self::EvalBuf => Self::EVAL_BUF,
			Self::Add => Self::ADD,
			Self::Sub => Self::SUB,
			Self::Mul => Self::MUL,
			Self::Div => Self::DIV,
			Self::Swp => Self::SWP,
			Self::Pow => Self::POW,
			Self::Root => Self::ROOT,
			Self::Exp => Self::EXP,
			Self::IntDiv => Self::INT_DIV,
			Self::Mod => Self::MOD,
			Self::Neg => Self::NEG,
			Self::Sqrt => Self::SQRT,
			Self::Sqr => Self::SQR,
			Self::Sin => Self::SIN,
			Self::Cos => Self::COS,
			Self::Tan => Self::TAN,
			Self::Asin => Self::ASIN,
			Self::Acos => Self::ACOS,
			Self::Atan => Self::ATAN,
			Self::Rad => Self::RAD,
			Self::Deg => Self::DEG,
			Self::Pop => Self::POP,
			Self::CalcStore => Self::CALC_STORE,
			Self::CalcDelete => Self::CALC_DELETE,
			Self::CalcRecall => Self::CALC_RECALL,
		}
	}

	/// Index of a variant for (de)serialization
	const fn index(self) -> u32 {
		// NOTE: I've tried to avoid duplicating constant values like this, but seeing as these
		// indices should only need to be duped to one other location, deduping would just bloat the
		// file. Because of that, changes here MUST be reflected in the TryFrom<u32> implementation!
		// Not that there's good reason to be changing these anyways.
		match self {
			Self::Quit => 0,
			Self::DelChar => 1,
			Self::EvalBuf => 2,
			Self::Add => 3,
			Self::Sub => 4,
			Self::Mul => 5,
			Self::Div => 6,
			Self::Swp => 7,
			Self::Pow => 8,
			Self::Root => 9,
			Self::Exp => 10,
			Self::IntDiv => 11,
			Self::Mod => 12,
			Self::Neg => 13,
			Self::Sqrt => 14,
			Self::Sqr => 15,
			Self::Sin => 16,
			Self::Cos => 17,
			Self::Tan => 18,
			Self::Asin => 19,
			Self::Acos => 20,
			Self::Atan => 21,
			Self::Rad => 22,
			Self::Deg => 23,
			Self::Pop => 24,
			Self::CalcStore => 25,
			Self::CalcDelete => 26,
			Self::CalcRecall => 27,
		}
	}
}
impl TryFrom<u32> for ParserCommandVariant {
	type Error = ();

	fn try_from(value: u32) -> Result<Self, Self::Error> {
		// NOTE: These values must stay in sync with the ParserCommandVariant::index() function
		// above!
	    match value {
			0 => Ok(Self::Quit),
			1 => Ok(Self::DelChar),
			2 => Ok(Self::EvalBuf),
			3 => Ok(Self::Add),
			4 => Ok(Self::Sub),
			5 => Ok(Self::Mul),
			6 => Ok(Self::Div),
			7 => Ok(Self::Swp),
			8 => Ok(Self::Pow),
			9 => Ok(Self::Root),
			10 => Ok(Self::Exp),
			11 => Ok(Self::IntDiv),
			12 => Ok(Self::Mod),
			13 => Ok(Self::Neg),
			14 => Ok(Self::Sqrt),
			15 => Ok(Self::Sqr),
			16 => Ok(Self::Sin),
			17 => Ok(Self::Cos),
			18 => Ok(Self::Tan),
			19 => Ok(Self::Asin),
			20 => Ok(Self::Acos),
			21 => Ok(Self::Atan),
			22 => Ok(Self::Rad),
			23 => Ok(Self::Deg),
			24 => Ok(Self::Pop),
			25 => Ok(Self::CalcStore),
			26 => Ok(Self::CalcDelete),
			27 => Ok(Self::CalcRecall),
			_ => Err(()),
		}
	}
}
impl TryFrom<&str> for ParserCommandVariant {
	type Error = ();

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		match value {
			Self::QUIT => Ok(Self::Quit),
			Self::DEL_CHAR => Ok(Self::DelChar),
			Self::EVAL_BUF => Ok(Self::EvalBuf),
			Self::ADD => Ok(Self::Add),
			Self::SUB => Ok(Self::Sub),
			Self::MUL => Ok(Self::Mul),
			Self::DIV => Ok(Self::Div),
			Self::SWP => Ok(Self::Swp),
			Self::POW => Ok(Self::Pow),
			Self::ROOT => Ok(Self::Root),
			Self::EXP => Ok(Self::Exp),
			Self::INT_DIV => Ok(Self::IntDiv),
			Self::MOD => Ok(Self::Mod),
			Self::NEG => Ok(Self::Neg),
			Self::SQRT => Ok(Self::Sqrt),
			Self::SQR => Ok(Self::Sqr),
			Self::SIN => Ok(Self::Sin),
			Self::COS => Ok(Self::Cos),
			Self::TAN => Ok(Self::Tan),
			Self::ASIN => Ok(Self::Asin),
			Self::ACOS => Ok(Self::Acos),
			Self::ATAN => Ok(Self::Atan),
			Self::RAD => Ok(Self::Rad),
			Self::DEG => Ok(Self::Deg),
			Self::POP => Ok(Self::Pop),
			Self::CALC_STORE => Ok(Self::CalcStore),
			Self::CALC_DELETE => Ok(Self::CalcDelete),
			Self::CALC_RECALL => Ok(Self::CalcRecall),
			 _ => Err(())
		}
	}
}
impl From<ParserCommand> for ParserCommandVariant {
	fn from(value: ParserCommand) -> Self {
		match value {
			ParserCommand::Quit => ParserCommandVariant::Quit,
			ParserCommand::DelChar => ParserCommandVariant::DelChar,
			ParserCommand::EvalBuf => ParserCommandVariant::EvalBuf,
			ParserCommand::CalcBinOp(BinOp::Add) => ParserCommandVariant::Add,
			ParserCommand::CalcBinOp(BinOp::Sub) => ParserCommandVariant::Sub,
			ParserCommand::CalcBinOp(BinOp::Mul) => ParserCommandVariant::Mul,
			ParserCommand::CalcBinOp(BinOp::Div) => ParserCommandVariant::Div,
			ParserCommand::CalcBinOp(BinOp::Swp) => ParserCommandVariant::Swp,
			ParserCommand::CalcBinOp(BinOp::Pow) => ParserCommandVariant::Pow,
			ParserCommand::CalcBinOp(BinOp::Root) => ParserCommandVariant::Root,
			ParserCommand::CalcBinOp(BinOp::Exp) => ParserCommandVariant::Exp,
			ParserCommand::CalcBinOp(BinOp::IntDiv) => ParserCommandVariant::IntDiv,
			ParserCommand::CalcBinOp(BinOp::Mod) => ParserCommandVariant::Mod,
			ParserCommand::CalcUnOp(UnOp::Neg) => ParserCommandVariant::Neg,
			ParserCommand::CalcUnOp(UnOp::Sqrt) => ParserCommandVariant::Sqrt,
			ParserCommand::CalcUnOp(UnOp::Sqr) => ParserCommandVariant::Sqr,
			ParserCommand::CalcUnOp(UnOp::Sin) => ParserCommandVariant::Sin,
			ParserCommand::CalcUnOp(UnOp::Cos) => ParserCommandVariant::Cos,
			ParserCommand::CalcUnOp(UnOp::Tan) => ParserCommandVariant::Tan,
			ParserCommand::CalcUnOp(UnOp::Asin) => ParserCommandVariant::Asin,
			ParserCommand::CalcUnOp(UnOp::Acos) => ParserCommandVariant::Acos,
			ParserCommand::CalcUnOp(UnOp::Atan) => ParserCommandVariant::Atan,
			ParserCommand::CalcUnOp(UnOp::Rad) => ParserCommandVariant::Rad,
			ParserCommand::CalcUnOp(UnOp::Deg) => ParserCommandVariant::Deg,
			ParserCommand::CalcUnOp(UnOp::Pop) => ParserCommandVariant::Pop,
			ParserCommand::CalcStore => ParserCommandVariant::CalcStore,
			ParserCommand::CalcDelete => ParserCommandVariant::CalcDelete,
			ParserCommand::CalcRecall => ParserCommandVariant::CalcRecall,
		}
	}
}
// TryFrom chosen here in the chance that commands eventually have data attached to them. The
// conversion will only be able to handle the unit variants.
impl TryFrom<ParserCommandVariant> for ParserCommand {
	type Error = ();

	fn try_from(value: ParserCommandVariant) -> Result<Self, Self::Error> {
	    match value {
			ParserCommandVariant::Quit => Ok(ParserCommand::Quit),
			ParserCommandVariant::DelChar => Ok(ParserCommand::DelChar),
			ParserCommandVariant::EvalBuf => Ok(ParserCommand::EvalBuf),
			ParserCommandVariant::Add => Ok(ParserCommand::CalcBinOp(BinOp::Add)),
			ParserCommandVariant::Sub => Ok(ParserCommand::CalcBinOp(BinOp::Sub)),
			ParserCommandVariant::Mul => Ok(ParserCommand::CalcBinOp(BinOp::Mul)),
			ParserCommandVariant::Div => Ok(ParserCommand::CalcBinOp(BinOp::Div)),
			ParserCommandVariant::Swp => Ok(ParserCommand::CalcBinOp(BinOp::Swp)),
			ParserCommandVariant::Pow => Ok(ParserCommand::CalcBinOp(BinOp::Pow)),
			ParserCommandVariant::Root => Ok(ParserCommand::CalcBinOp(BinOp::Root)),
			ParserCommandVariant::Exp => Ok(ParserCommand::CalcBinOp(BinOp::Exp)),
			ParserCommandVariant::IntDiv => Ok(ParserCommand::CalcBinOp(BinOp::IntDiv)),
			ParserCommandVariant::Mod => Ok(ParserCommand::CalcBinOp(BinOp::Mod)),
			ParserCommandVariant::Neg => Ok(ParserCommand::CalcUnOp(UnOp::Neg)),
			ParserCommandVariant::Sqrt => Ok(ParserCommand::CalcUnOp(UnOp::Sqrt)),
			ParserCommandVariant::Sqr => Ok(ParserCommand::CalcUnOp(UnOp::Sqr)),
			ParserCommandVariant::Sin => Ok(ParserCommand::CalcUnOp(UnOp::Sin)),
			ParserCommandVariant::Cos => Ok(ParserCommand::CalcUnOp(UnOp::Cos)),
			ParserCommandVariant::Tan => Ok(ParserCommand::CalcUnOp(UnOp::Tan)),
			ParserCommandVariant::Asin => Ok(ParserCommand::CalcUnOp(UnOp::Asin)),
			ParserCommandVariant::Acos => Ok(ParserCommand::CalcUnOp(UnOp::Acos)),
			ParserCommandVariant::Atan => Ok(ParserCommand::CalcUnOp(UnOp::Atan)),
			ParserCommandVariant::Rad => Ok(ParserCommand::CalcUnOp(UnOp::Rad)),
			ParserCommandVariant::Deg => Ok(ParserCommand::CalcUnOp(UnOp::Deg)),
			ParserCommandVariant::Pop => Ok(ParserCommand::CalcUnOp(UnOp::Pop)),
			ParserCommandVariant::CalcStore => Ok(ParserCommand::CalcStore),
			ParserCommandVariant::CalcDelete => Ok(ParserCommand::CalcDelete),
			ParserCommandVariant::CalcRecall => Ok(ParserCommand::CalcRecall),
		}
	}
}

struct ParserCommandVariantVisitor;
impl<'de> Visitor<'de> for ParserCommandVariantVisitor {
	type Value = ParserCommandVariant;

	fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(formatter, "a valid parser command variant")
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		if let Ok(v) = ParserCommandVariant::try_from(v) {
			Ok(v)
		} else {
			Err(E::unknown_variant(v, ParserCommandVariant::ALL_NAMES))
		}
	}

	fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		if v > u32::MAX as i64 || v < 0 {
			Err(E::invalid_value(Unexpected::Other("integer outside bounds of u32"), &"integer fitting in u32"))
		} else {
			self.visit_u32(v as u32)
		}
	}

	fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		if v > u32::MAX as i128 || v < 0 {
			Err(E::invalid_value(Unexpected::Other("integer outside bounds of u32"), &"integer fitting in u32"))
		} else {
			self.visit_u32(v as u32)
		}
	}

	fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		if let Ok(v) = ParserCommandVariant::try_from(v) {
			Ok(v)
		} else {
			Err(E::invalid_value(Unexpected::Other("an unrecognized integer discriminant"), &"an integer matching an enum variant's index"))
		}
	}

	fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		if v > u32::MAX as u64 {
			Err(E::invalid_value(Unexpected::Other("integer outside bounds of u64"), &"integer fitting in u32"))
		} else {
			self.visit_u32(v as u32)
		}
	}

	fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		if v > u32::MAX as u128 {
			Err(E::invalid_value(Unexpected::Other("integer outside bounds of u64"), &"integer fitting in u32"))
		} else {
			self.visit_u32(v as u32)
		}
	}
}
impl<'de> Deserialize<'de> for ParserCommandVariant {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>
	{
		Ok(deserializer.deserialize_identifier(ParserCommandVariantVisitor)?)
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
    // more than one action to be performed in a single press, hence the need for `Vec`

    // Maybe return an `Option` rather than an empty `Vec`?
    pub fn parse(&mut self, ke: KeyEvent) -> Vec<ExternalCommand> {
        let mut actions = Vec::new();

        // I don't know a better way to return early on non-presses
        match ke.kind {
            KeyEventKind::Press => (),
            _ => return actions,
        }

        if let Some(inc_cmd) = self.config.immediate_cmds.0.get(&ke.code) {
            // Special cases, no pre-buffer evaluation should be done
            match *inc_cmd {
                ParserCommand::DelChar => {
                    self.bfr.pop();
                    return actions;
                }
                ParserCommand::EvalBuf => {
                    if let Some(cmd) = self.eval_buffer(false) {
                        actions.push(cmd);
                        self.bfr.clear();
                    }
                    return actions;
                }
                _ => {
                    // If not a special case, do pre-eval
                    if let Some(cmd) = self.eval_buffer(true) {
                        actions.push(cmd);
                        self.bfr.clear();
                    }
                }
            }

            match *inc_cmd {
                ParserCommand::Quit => actions.push(ExternalCommand::Quit),

                ParserCommand::CalcBinOp(op) => {
                    actions.push(ExternalCommand::CalcCmd(Command::BinOp(op)))
                }
                ParserCommand::CalcUnOp(op) => {
                    actions.push(ExternalCommand::CalcCmd(Command::UnOp(op)))
                }
                ParserCommand::CalcStore => {
					actions.push(ExternalCommand::CalcCmd(Command::Sto(self.bfr.iter().collect())));
					self.bfr.clear();
                }
                ParserCommand::CalcDelete => {
					self.bfr.clear();
					actions.push(ExternalCommand::CalcCmd(Command::Del(self.bfr.iter().collect())));
                }
                ParserCommand::CalcRecall => {
					self.bfr.clear();
					actions.push(ExternalCommand::CalcCmd(Command::Rcl(self.bfr.iter().collect())));
                }

				// Should always be handled by the prior match!
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
            return Some(ExternalCommand::CalcCmd(Command::Push(None)));
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
            }
            EvalMode::Commands => {
                if let Some(inc_cmd) = self.config.string_cmds.get(bfr_string.as_str()) {
                    match *inc_cmd {
                        ParserCommand::Quit => return Some(ExternalCommand::Quit),

                        ParserCommand::CalcBinOp(op) => {
                            return Some(ExternalCommand::CalcCmd(Command::BinOp(op)))
                        }
                        ParserCommand::CalcUnOp(op) => {
                            return Some(ExternalCommand::CalcCmd(Command::UnOp(op)))
                        }

                        _ => panic!("Invalid command in eval_buffer"),
                    }
                } else {
                    return None;
                }
            }
            EvalMode::All => {
                if let Ok(v) = num_parse {
                    return Some(ExternalCommand::CalcCmd(Command::Push(Some(v))));
                } else if let Some(inc_cmd) = self.config.string_cmds.get(bfr_string.as_str()) {
                    match *inc_cmd {
                        ParserCommand::Quit => return Some(ExternalCommand::Quit),

                        ParserCommand::CalcBinOp(op) => {
                            return Some(ExternalCommand::CalcCmd(Command::BinOp(op)))
                        }
                        ParserCommand::CalcUnOp(op) => {
                            return Some(ExternalCommand::CalcCmd(Command::UnOp(op)))
                        }

                        _ => panic!("Invalid command in eval_buffer"),
                    }
                } else {
                    return None;
                }
            }
        }
    }
}
