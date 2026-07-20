mod calculator;
mod tui;

use calculator::{Calculator, CalculatorConfig};
use tui::{TUIConfig, TUI};

fn main() {
	// TODO: read configs from file to create config structs
    let calc = Calculator::new(CalculatorConfig::default());

    let mut application = TUI::new(calc, TUIConfig::default());
    // TODO: `unwrap` should not be used, the return should be looked at and an appropriate error
    // printed where applicable
    application.run().unwrap();
}
