#[derive(Debug)]
pub enum Command {
    AppendToBfr(char),
	RemoveFromBfr,
	ClearBfr,
    BinOp(BinOp),
	UnOp(UnOp),
	RotateIn(Option<f64>),
    Exit,
    NoOp,
	Sto(char),
	Rcl(char),
	Del(char),
}
#[derive(Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
	Swp,
	Pow,
	Rt,
	Exp, // 10^x
	IntDiv,
	Mod,
}
#[derive(Debug)]
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