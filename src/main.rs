mod application;
mod command;
mod tui_windows;

use application::{Calculator, Memory, Response};
use tui_windows::*;

use crossterm::execute;
use crossterm::cursor::{position, Hide, Show};
use crossterm::event::read;
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, size, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};

use std::cell::RefCell;

fn main() -> Result<(), std::io::Error> {
    enable_raw_mode()?;

    let memory = RefCell::new(Memory::new());
    let calc = RefCell::new(Calculator::new(&memory));
    let window_calc = Window::new(&calc, WindowConfig { rel_size: 1.0, wrapping: true });
    let window_mem = Window::new(&memory, WindowConfig { rel_size: 1.1, wrapping: true });
    let app = Container::new(vec!(TileType::Window(window_calc), TileType::Window(window_mem)), false, 1.0);
    
    let mut out = std::io::stdout();
    execute!(out, EnterAlternateScreen, Clear(ClearType::All), Hide)?;
    loop {
        let (cols, rows) = size()?;
        let (cur_col, cur_row) = position()?;

        app.draw(&mut out, (cur_row, cur_col), (0, 0), (rows, cols))?;
        
        let mut calc_borrow = calc.borrow_mut();
        let code = calc_borrow.process_event(read().unwrap())?;
        match code {
            Response::NoAction => (),
            Response::Exit => break,
        }
    }

    execute!(std::io::stdout(), LeaveAlternateScreen, Show)?;
    disable_raw_mode()?;
    Ok(())
}
