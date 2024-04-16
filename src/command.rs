pub enum Command {
    AppendToBfr(char),
	RemoveFromBfr,
	ClearBfr,
    BinOp(BinOp),
	UnOp(UnOp),
	RotateIn(Option<f64>),
	Draw,
    Exit,
    NoOp,
}
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
	Swp,
	Pow,
	Rt,
	Exp, // 10^x
}
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