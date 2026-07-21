mod calculator;
mod tui;

use serde::{Deserialize, Serialize};

use calculator::{Calculator, CalculatorConfig};
use tui::{TUIConfig, TUI};

#[derive(Serialize, Deserialize, Default)]
struct Config {
	calc: CalculatorConfig,
	tui: TUIConfig,
}

fn main() {
	// TODO: read configs from file to create config structs
	let config = Config::default();

    let calc = Calculator::new(config.calc);

    let mut application = TUI::new(calc, config.tui);
    // TODO: `unwrap` should not be used, the return should be looked at and an appropriate error
    // printed where applicable
    application.run().unwrap();
}
