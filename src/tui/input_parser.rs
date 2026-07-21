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
// Deriving serialize/deserialize maps the data structure in a dissatisfactory manner
impl Serialize for ParserCommand {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
	    S: Serializer
	{
		match self {
			Self::Quit => serializer.serialize_unit_variant("ParserCommand", 0, "Quit"),
			Self::DelChar => serializer.serialize_unit_variant("ParserCommand", 1, "DelChar"),
			Self::EvalBuf => serializer.serialize_unit_variant("ParserCommand", 2, "EvalBuf"),

			// Deriving serialize creates undesired nesting here, so we flatten it out manually
			// Supposedly there's an attribute that can do this automatically but it didn't seem to
			// work
			Self::CalcBinOp(op) => match op {
				BinOp::Add => serializer.serialize_unit_variant("ParserCommand", 3, "Add"),
				BinOp::Sub => serializer.serialize_unit_variant("ParserCommand", 4, "Sub"),
				BinOp::Mul => serializer.serialize_unit_variant("ParserCommand", 5, "Mul"),
				BinOp::Div => serializer.serialize_unit_variant("ParserCommand", 6, "Div"),
				BinOp::Swp => serializer.serialize_unit_variant("ParserCommand", 7, "Swp"),
				BinOp::Pow => serializer.serialize_unit_variant("ParserCommand", 8, "Pow"),
				BinOp::Root => serializer.serialize_unit_variant("ParserCommand", 9, "Root"),
				BinOp::Exp => serializer.serialize_unit_variant("ParserCommand", 10, "Exp"),
				BinOp::IntDiv => serializer.serialize_unit_variant("ParserCommand", 11, "IntDiv"),
				BinOp::Mod => serializer.serialize_unit_variant("ParserCommand", 12, "Mod"),
			},
			Self::CalcUnOp(op) => match op {
				UnOp::Neg => serializer.serialize_unit_variant("ParserCommand", 13, "Neg"),
				UnOp::Sqrt => serializer.serialize_unit_variant("ParserCommand", 14, "Sqrt"),
				UnOp::Sqr => serializer.serialize_unit_variant("ParserCommand", 15, "Sqr"),
				UnOp::Sin => serializer.serialize_unit_variant("ParserCommand", 16, "Sin"),
				UnOp::Cos => serializer.serialize_unit_variant("ParserCommand", 17, "Cos"),
				UnOp::Tan => serializer.serialize_unit_variant("ParserCommand", 18, "Tan"),
				UnOp::Asin => serializer.serialize_unit_variant("ParserCommand", 19, "Asin"),
				UnOp::Acos => serializer.serialize_unit_variant("ParserCommand", 20, "Acos"),
				UnOp::Atan => serializer.serialize_unit_variant("ParserCommand", 21, "Atan"),
				UnOp::Rad => serializer.serialize_unit_variant("ParserCommand", 22, "Rad"),
				UnOp::Deg => serializer.serialize_unit_variant("ParserCommand", 23, "Deg"),
				UnOp::Pop => serializer.serialize_unit_variant("ParserCommand", 24, "Pop"),
			},
			Self::CalcStore => serializer.serialize_unit_variant("ParserCommand", 25, "CalcStore"),
			Self::CalcDelete => serializer.serialize_unit_variant("ParserCommand", 26, "CalcDelete"),
			Self::CalcRecall => serializer.serialize_unit_variant("ParserCommand", 27, "CalcRecall"),
		}
	}
}
impl<'de> Deserialize<'de> for ParserCommand {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
	    D: Deserializer<'de>
	{
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
				match v {
					"Quit" => Ok(ParserCommand::Quit),
					"DelChar" => Ok(ParserCommand::DelChar),
					"EvalBuf" => Ok(ParserCommand::EvalBuf),
					"Add" => Ok(ParserCommand::CalcBinOp(BinOp::Add)),
					"Sub" => Ok(ParserCommand::CalcBinOp(BinOp::Sub)),
					"Mul" => Ok(ParserCommand::CalcBinOp(BinOp::Mul)),
					"Div" => Ok(ParserCommand::CalcBinOp(BinOp::Div)),
					"Swp" => Ok(ParserCommand::CalcBinOp(BinOp::Swp)),
					"Pow" => Ok(ParserCommand::CalcBinOp(BinOp::Pow)),
					"Root" => Ok(ParserCommand::CalcBinOp(BinOp::Root)),
					"Exp" => Ok(ParserCommand::CalcBinOp(BinOp::Exp)),
					"IntDiv" => Ok(ParserCommand::CalcBinOp(BinOp::IntDiv)),
					"Mod" => Ok(ParserCommand::CalcBinOp(BinOp::Mod)),
					"Neg" => Ok(ParserCommand::CalcUnOp(UnOp::Neg)),
					"Sqrt" => Ok(ParserCommand::CalcUnOp(UnOp::Sqrt)),
					"Sqr" => Ok(ParserCommand::CalcUnOp(UnOp::Sqr)),
					"Sin" => Ok(ParserCommand::CalcUnOp(UnOp::Sin)),
					"Cos" => Ok(ParserCommand::CalcUnOp(UnOp::Cos)),
					"Tan" => Ok(ParserCommand::CalcUnOp(UnOp::Tan)),
					"Asin" => Ok(ParserCommand::CalcUnOp(UnOp::Asin)),
					"Acos" => Ok(ParserCommand::CalcUnOp(UnOp::Acos)),
					"Atan" => Ok(ParserCommand::CalcUnOp(UnOp::Atan)),
					"Rad" => Ok(ParserCommand::CalcUnOp(UnOp::Rad)),
					"Deg" => Ok(ParserCommand::CalcUnOp(UnOp::Deg)),
					"Pop" => Ok(ParserCommand::CalcUnOp(UnOp::Pop)),
					"CalcStore" => Ok(ParserCommand::CalcStore),
					"CalcDelete" => Ok(ParserCommand::CalcDelete),
					"CalcRecall" => Ok(ParserCommand::CalcRecall),
					_ => Err(E::unknown_variant(v, &[ // TODO: make only one copy of this array
						"Quit",
						"DelChar",
						"EvalBuf",
						"Add",
						"Sub",
						"Mul",
						"Div",
						"Swp",
						"Pow",
						"Root",
						"Exp",
						"IntDiv",
						"Mod",
						"Neg",
						"Sqrt",
						"Sqr",
						"Sin",
						"Cos",
						"Tan",
						"Asin",
						"Acos",
						"Atan",
						"Rad",
						"Deg",
						"Pop",
						"CalcStore",
						"CalcDelete",
						"CalcRecall",
					])),
				}
			}

			fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
			where
			    E: serde::de::Error,
			{
				if v < 0 {
					Err(E::invalid_value(Unexpected::Other("integer outside bounds of u64"), &"integer in the range [0, 27]"))
				} else {
					self.visit_u64(v as u64)
				}
			}

			fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
			where
			    E: serde::de::Error,
			{
				if v > u64::MAX as i128 || v < 0 {
					Err(E::invalid_value(Unexpected::Other("integer outside bounds of u64"), &"integer in the range [0, 27]"))
				} else {
					self.visit_u64(v as u64)
				}
			}

			fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
			where
			    E: serde::de::Error,
			{
				match v {
					0 => Ok(ParserCommand::Quit),
					1 => Ok(ParserCommand::DelChar),
					2 => Ok(ParserCommand::EvalBuf),
					3 => Ok(ParserCommand::CalcBinOp(BinOp::Add)),
					4 => Ok(ParserCommand::CalcBinOp(BinOp::Sub)),
					5 => Ok(ParserCommand::CalcBinOp(BinOp::Mul)),
					6 => Ok(ParserCommand::CalcBinOp(BinOp::Div)),
					7 => Ok(ParserCommand::CalcBinOp(BinOp::Swp)),
					8 => Ok(ParserCommand::CalcBinOp(BinOp::Pow)),
					9 => Ok(ParserCommand::CalcBinOp(BinOp::Root)),
					10 => Ok(ParserCommand::CalcBinOp(BinOp::Exp)),
					11 => Ok(ParserCommand::CalcBinOp(BinOp::IntDiv)),
					12 => Ok(ParserCommand::CalcBinOp(BinOp::Mod)),
					13 => Ok(ParserCommand::CalcUnOp(UnOp::Neg)),
					14 => Ok(ParserCommand::CalcUnOp(UnOp::Sqrt)),
					15 => Ok(ParserCommand::CalcUnOp(UnOp::Sqr)),
					16 => Ok(ParserCommand::CalcUnOp(UnOp::Sin)),
					17 => Ok(ParserCommand::CalcUnOp(UnOp::Cos)),
					18 => Ok(ParserCommand::CalcUnOp(UnOp::Tan)),
					19 => Ok(ParserCommand::CalcUnOp(UnOp::Asin)),
					20 => Ok(ParserCommand::CalcUnOp(UnOp::Acos)),
					21 => Ok(ParserCommand::CalcUnOp(UnOp::Atan)),
					22 => Ok(ParserCommand::CalcUnOp(UnOp::Rad)),
					23 => Ok(ParserCommand::CalcUnOp(UnOp::Deg)),
					24 => Ok(ParserCommand::CalcUnOp(UnOp::Pop)),
					25 => Ok(ParserCommand::CalcStore),
					26 => Ok(ParserCommand::CalcDelete),
					27 => Ok(ParserCommand::CalcRecall),
					_ => Err(E::invalid_value(Unexpected::Unsigned(v), &"integer in the range [0, 27]")),
				}
			}

			fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
			where
			    E: serde::de::Error,
			{
				if v > u64::MAX as u128 {
					Err(E::invalid_value(Unexpected::Other("integer outside bounds of u64"), &"integer in the range [0, 27]"))
				} else {
					self.visit_u64(v as u64)
				}
			}

			// i dont understand this function at all and the docs are painfully sparse about it
			fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
			where
				A: serde::de::EnumAccess<'de>,
			{
				// i *think* i need to provide a visitor capable of distinguishing enum variants
				// (probalby using the other methods already implemented) and then signal to the
				// deserializer what kind of data the enum holds based on the variant, but since
				// this enum only has unit variants, i should be able to take some shortcuts here
				// (notably by using THIS visitor instead of a standalone variant detector visitor)
				//
				// yea no that just overflows the stack ok
				// but i still think the other functions are useful so i'll just move or copy them
				//
				// fuck it i'm copying it all into another visitor idc anymore

				enum ParserCommandType {
					Quit,
					DelChar,
					EvalBuf,
					CalcBinOpAdd,
					CalcBinOpSub,
					CalcBinOpMul,
					CalcBinOpDiv,
					CalcBinOpSwp,
					CalcBinOpPow,
					CalcBinOpRoot,
					CalcBinOpExp,
					CalcBinOpIntDiv,
					CalcBinOpMod,
					CalcUnOpNeg,
					CalcUnOpSqrt,
					CalcUnOpSqr,
					CalcUnOpSin,
					CalcUnOpCos,
					CalcUnOpTan,
					CalcUnOpAsin,
					CalcUnOpAcos,
					CalcUnOpAtan,
					CalcUnOpRad,
					CalcUnOpDeg,
					CalcUnOpPop,
					CalcStore,
					CalcDelete,
					CalcRecall,
				}
				impl<'de> Deserialize<'de> for ParserCommandType {
					fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
					where
					    D: Deserializer<'de>
					{
						struct ParserCommandTypeVisitor;
						impl<'de> Visitor<'de> for ParserCommandTypeVisitor {
							type Value = ParserCommandType;

							fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
								write!(formatter, "a valid parser command")
							}

							fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
							where
								E: serde::de::Error,
							{
								match v {
									"Quit" => Ok(ParserCommandType::Quit),
									"DelChar" => Ok(ParserCommandType::DelChar),
									"EvalBuf" => Ok(ParserCommandType::EvalBuf),
									"Add" => Ok(ParserCommandType::CalcBinOpAdd),
									"Sub" => Ok(ParserCommandType::CalcBinOpSub),
									"Mul" => Ok(ParserCommandType::CalcBinOpMul),
									"Div" => Ok(ParserCommandType::CalcBinOpDiv),
									"Swp" => Ok(ParserCommandType::CalcBinOpSwp),
									"Pow" => Ok(ParserCommandType::CalcBinOpPow),
									"Root" => Ok(ParserCommandType::CalcBinOpRoot),
									"Exp" => Ok(ParserCommandType::CalcBinOpExp),
									"IntDiv" => Ok(ParserCommandType::CalcBinOpIntDiv),
									"Mod" => Ok(ParserCommandType::CalcBinOpMod),
									"Neg" => Ok(ParserCommandType::CalcUnOpNeg),
									"Sqrt" => Ok(ParserCommandType::CalcUnOpSqrt),
									"Sqr" => Ok(ParserCommandType::CalcUnOpSqr),
									"Sin" => Ok(ParserCommandType::CalcUnOpSin),
									"Cos" => Ok(ParserCommandType::CalcUnOpCos),
									"Tan" => Ok(ParserCommandType::CalcUnOpTan),
									"Asin" => Ok(ParserCommandType::CalcUnOpAsin),
									"Acos" => Ok(ParserCommandType::CalcUnOpAcos),
									"Atan" => Ok(ParserCommandType::CalcUnOpAtan),
									"Rad" => Ok(ParserCommandType::CalcUnOpRad),
									"Deg" => Ok(ParserCommandType::CalcUnOpDeg),
									"Pop" => Ok(ParserCommandType::CalcUnOpPop),
									"CalcStore" => Ok(ParserCommandType::CalcStore),
									"CalcDelete" => Ok(ParserCommandType::CalcDelete),
									"CalcRecall" => Ok(ParserCommandType::CalcRecall),
									_ => Err(E::unknown_variant(v, &[ // TODO: make only one copy of this array
										"Quit",
										"DelChar",
										"EvalBuf",
										"Add",
										"Sub",
										"Mul",
										"Div",
										"Swp",
										"Pow",
										"Root",
										"Exp",
										"IntDiv",
										"Mod",
										"Neg",
										"Sqrt",
										"Sqr",
										"Sin",
										"Cos",
										"Tan",
										"Asin",
										"Acos",
										"Atan",
										"Rad",
										"Deg",
										"Pop",
										"CalcStore",
										"CalcDelete",
										"CalcRecall",
									])),
								}
							}

							fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
							where
								E: serde::de::Error,
							{
								if v < 0 {
									Err(E::invalid_value(Unexpected::Other("integer outside bounds of u64"), &"integer in the range [0, 27]"))
								} else {
									self.visit_u64(v as u64)
								}
							}

							fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
							where
								E: serde::de::Error,
							{
								if v > u64::MAX as i128 || v < 0 {
									Err(E::invalid_value(Unexpected::Other("integer outside bounds of u64"), &"integer in the range [0, 27]"))
								} else {
									self.visit_u64(v as u64)
								}
							}

							fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
							where
								E: serde::de::Error,
							{
								match v {
									0 => Ok(ParserCommandType::Quit),
									1 => Ok(ParserCommandType::DelChar),
									2 => Ok(ParserCommandType::EvalBuf),
									3 => Ok(ParserCommandType::CalcBinOpAdd),
									4 => Ok(ParserCommandType::CalcBinOpSub),
									5 => Ok(ParserCommandType::CalcBinOpMul),
									6 => Ok(ParserCommandType::CalcBinOpDiv),
									7 => Ok(ParserCommandType::CalcBinOpSwp),
									8 => Ok(ParserCommandType::CalcBinOpPow),
									9 => Ok(ParserCommandType::CalcBinOpRoot),
									10 => Ok(ParserCommandType::CalcBinOpExp),
									11 => Ok(ParserCommandType::CalcBinOpIntDiv),
									12 => Ok(ParserCommandType::CalcBinOpMod),
									13 => Ok(ParserCommandType::CalcUnOpNeg),
									14 => Ok(ParserCommandType::CalcUnOpSqrt),
									15 => Ok(ParserCommandType::CalcUnOpSqr),
									16 => Ok(ParserCommandType::CalcUnOpSin),
									17 => Ok(ParserCommandType::CalcUnOpCos),
									18 => Ok(ParserCommandType::CalcUnOpTan),
									19 => Ok(ParserCommandType::CalcUnOpAsin),
									20 => Ok(ParserCommandType::CalcUnOpAcos),
									21 => Ok(ParserCommandType::CalcUnOpAtan),
									22 => Ok(ParserCommandType::CalcUnOpRad),
									23 => Ok(ParserCommandType::CalcUnOpDeg),
									24 => Ok(ParserCommandType::CalcUnOpPop),
									25 => Ok(ParserCommandType::CalcStore),
									26 => Ok(ParserCommandType::CalcDelete),
									27 => Ok(ParserCommandType::CalcRecall),
									_ => Err(E::invalid_value(Unexpected::Unsigned(v), &"integer in the range [0, 27]")),
								}
							}

							fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
							where
								E: serde::de::Error,
							{
								if v > u64::MAX as u128 {
									Err(E::invalid_value(Unexpected::Other("integer outside bounds of u64"), &"integer in the range [0, 27]"))
								} else {
									self.visit_u64(v as u64)
								}
							}
						}

						Ok(deserializer.deserialize_identifier(ParserCommandTypeVisitor)?)
					}
				}

				let (result, accessor) = data.variant::<ParserCommandType>()?;
				accessor.unit_variant()?;
				match result {
					ParserCommandType::Quit => Ok(ParserCommand::Quit),
					ParserCommandType::DelChar => Ok(ParserCommand::DelChar),
					ParserCommandType::EvalBuf => Ok(ParserCommand::EvalBuf),
					ParserCommandType::CalcBinOpAdd => Ok(ParserCommand::CalcBinOp(BinOp::Add)),
					ParserCommandType::CalcBinOpSub => Ok(ParserCommand::CalcBinOp(BinOp::Sub)),
					ParserCommandType::CalcBinOpMul => Ok(ParserCommand::CalcBinOp(BinOp::Mul)),
					ParserCommandType::CalcBinOpDiv => Ok(ParserCommand::CalcBinOp(BinOp::Div)),
					ParserCommandType::CalcBinOpSwp => Ok(ParserCommand::CalcBinOp(BinOp::Swp)),
					ParserCommandType::CalcBinOpPow => Ok(ParserCommand::CalcBinOp(BinOp::Pow)),
					ParserCommandType::CalcBinOpRoot => Ok(ParserCommand::CalcBinOp(BinOp::Root)),
					ParserCommandType::CalcBinOpExp => Ok(ParserCommand::CalcBinOp(BinOp::Exp)),
					ParserCommandType::CalcBinOpIntDiv => Ok(ParserCommand::CalcBinOp(BinOp::IntDiv)),
					ParserCommandType::CalcBinOpMod => Ok(ParserCommand::CalcBinOp(BinOp::Mod)),
					ParserCommandType::CalcUnOpNeg => Ok(ParserCommand::CalcUnOp(UnOp::Neg)),
					ParserCommandType::CalcUnOpSqrt => Ok(ParserCommand::CalcUnOp(UnOp::Sqrt)),
					ParserCommandType::CalcUnOpSqr => Ok(ParserCommand::CalcUnOp(UnOp::Sqr)),
					ParserCommandType::CalcUnOpSin => Ok(ParserCommand::CalcUnOp(UnOp::Sin)),
					ParserCommandType::CalcUnOpCos => Ok(ParserCommand::CalcUnOp(UnOp::Cos)),
					ParserCommandType::CalcUnOpTan => Ok(ParserCommand::CalcUnOp(UnOp::Tan)),
					ParserCommandType::CalcUnOpAsin => Ok(ParserCommand::CalcUnOp(UnOp::Asin)),
					ParserCommandType::CalcUnOpAcos => Ok(ParserCommand::CalcUnOp(UnOp::Acos)),
					ParserCommandType::CalcUnOpAtan => Ok(ParserCommand::CalcUnOp(UnOp::Atan)),
					ParserCommandType::CalcUnOpRad => Ok(ParserCommand::CalcUnOp(UnOp::Rad)),
					ParserCommandType::CalcUnOpDeg => Ok(ParserCommand::CalcUnOp(UnOp::Deg)),
					ParserCommandType::CalcUnOpPop => Ok(ParserCommand::CalcUnOp(UnOp::Pop)),
					ParserCommandType::CalcStore => Ok(ParserCommand::CalcStore),
					ParserCommandType::CalcDelete => Ok(ParserCommand::CalcDelete),
					ParserCommandType::CalcRecall => Ok(ParserCommand::CalcRecall),
				}
			}
		}

		deserializer.deserialize_enum(
			"ParserCommand",
			&[
				"Quit",
				"DelChar",
				"EvalBuf",
				"Add",
				"Sub",
				"Mul",
				"Div",
				"Swp",
				"Pow",
				"Root",
				"Exp",
				"IntDiv",
				"Mod",
				"Neg",
				"Sqrt",
				"Sqr",
				"Sin",
				"Cos",
				"Tan",
				"Asin",
				"Acos",
				"Atan",
				"Rad",
				"Deg",
				"Pop",
				"CalcStore",
				"CalcDelete",
				"CalcRecall",
			],
			ParserCommandVisitor
		)
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
