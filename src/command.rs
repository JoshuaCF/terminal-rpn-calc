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
	Pwr,
	Rt,
}
pub enum UnOp {
	Neg,
	Sqrt,
	Sqr,
}