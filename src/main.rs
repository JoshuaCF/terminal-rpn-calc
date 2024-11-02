mod calculator;
mod tui;

use calculator::Calculator;
use tui::TUI;

fn main() {
	// TODO: Make stack size configurable
	let calc = Calculator::new(12);

	let mut application = TUI::new(calc);
	// TODO: `unwrap` should not be used, the return should be looked at and an appropriate error
	// printed where applicable
	application.run().unwrap();
}
