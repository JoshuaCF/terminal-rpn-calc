pub enum Command {
    AppendToBfr(char),
	RemoveFromBfr,
	ClearBfr,
    BinOp(BinOp),
	UnOp(UnOp),
	RotIn(Option<f64>),
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
}
pub enum UnOp {
	Neg,
}