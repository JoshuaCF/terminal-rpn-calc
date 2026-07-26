mod calculator;
mod tui;

use std::env;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use clap::Parser;
use serde::{Deserialize, Serialize};

use calculator::{Calculator, CalculatorConfig};
use tui::{TUIConfig, TUI};

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct Config {
	calc: CalculatorConfig,
	tui: TUIConfig,
}

const APP_FOLDER: &'static str = "rpn_calc";
const APP_CONFIG_FILE: &'static str = "config.toml";

#[cfg(target_os = "linux")]
fn get_config_folder() -> Option<PathBuf> {
	// For Linux, I'm following the freedesktop spec
	// First, try XDG_CONFIG_HOME
	if let Some(v) = env::var_os("XDG_CONFIG_HOME") {
		return Some(v.into());
	}

	// If unavailable, try XDG_HOME/.config
	if let Some(v) = env::var_os("XDG_HOME") {
		let mut path: PathBuf = v.into();
		path.push(".config");
		return Some(path);
	}

	// Final attempt, HOME/.config
	if let Some(v) = env::var_os("HOME") {
		let mut path: PathBuf = v.into();
		path.push(".config");
		return Some(path);
	}

	// Otherwise, return none
	None
}

#[cfg(target_os = "windows")]
fn get_config_folder() -> Option<PathBuf> {
	// For Windows, I'm looking for %APPDATA%
	if let Some(v) = env::var_os("APPDATA") {
		return Some(v.into());
	}
	// Otherwise, return none
	None
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn get_config_folder() -> Option<PathBuf> {
	None
}

fn get_config_path() -> std::io::Result<PathBuf> {
	// TODO: If the config location was passed as an argument use that
	// Otherwise, try to find the platform-appropriate directory for configuration
	if let Some(mut v) = get_config_folder() {
		v.push(APP_FOLDER);
		v.push(APP_CONFIG_FILE);
		return Ok(v);
	}

	// If not found or inaccessible, just try creating a config file in the same folder as the executable
	// env::current_exe() should never return a value where this unwrap could fail
	let mut alt_location = env::current_exe()?.parent().unwrap().to_path_buf();
	alt_location.push(APP_CONFIG_FILE);
	Ok(alt_location.canonicalize()?)
}

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
	/// The path to load a config file from or generate a new config file to
	#[arg(short, long)]
	config_path: Option<String>,
	/// Generate a fresh config file at the location without prompting for confirmation, overwriting an existing file if one is there. Requires
	/// --config-path to be supplied
	#[arg(short, long)]
	generate_config: bool,
}

fn main() -> std::io::Result<()> {
	let args = Args::parse();

	if args.generate_config && args.config_path.is_none() {
		eprintln!("--config-path must be supplied when --generate-config is given");
		return Ok(());
	}

	let config_path;
	if let Some(p) = args.config_path {
		config_path = PathBuf::from(p);
	} else {
		config_path = match get_config_path() {
			Ok(v) => v,
			Err(e) => {
				eprintln!("Unable to deterministically find a default config path, please provide one as an argument to the executable with --config-path");
				eprintln!("Reason: {}", e);
				return Err(e);
			},
		};
	}

	let config: Config;
	// If a path is found but no config is there, prompt the user if they want to use that path. If
	// not, exit and inform them to specify a path as a flag.
	if !fs::exists(&config_path)? || args.generate_config {
		// prompt the user if the flag doesn't force config generation
		if !args.generate_config {
			println!("Create new config at '{}' (y/n)", config_path.display());
			let mut bfr = String::new();
			loop {
				std::io::stdin().read_line(&mut bfr)?;
				match bfr.trim() {
					"y" | "Y" => break,
					"n" | "N" => {
						eprintln!("Specify a config file through the flag to use the program.");
						return Ok(());
					},
					_ => bfr.clear(),
				}
			}
		}
		config = Config::default();
		// Create parent directory
		if let Some(v) = config_path.parent() {
			fs::create_dir_all(v)?;
		}

		let mut config_file = File::create(&config_path)?;
		// If the unwrap errors here, it's an error in my code to be fixed
		config_file.write(toml::to_string_pretty(&config).unwrap().as_bytes())?;
		config_file.flush()?;
	} else {
		let mut config_file = File::open(&config_path)?;
		let mut file_bfr = String::new();
		config_file.read_to_string(&mut file_bfr)?;

		config = match toml::from_str(file_bfr.as_str()) {
			Ok(v) => v,
			Err(e) => {
				eprintln!("Unable to parse config file");
				eprintln!("{}", e);
				return Ok(());
			},
		};
	}

	let calc = Calculator::new(config.calc);

	let mut application = TUI::new(calc, config.tui);
	// TODO: `unwrap` should not be used, the return should be looked at and an appropriate error
	// printed where applicable
	application.run().unwrap();
	Ok(())
}
