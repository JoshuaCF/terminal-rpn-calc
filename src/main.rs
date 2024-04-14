use crossterm::execute;
use crossterm::cursor::{position, MoveToColumn, MoveToNextLine};
use crossterm::event::{read, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{enable_raw_mode, size, ScrollUp};

use std::io::Write;

fn main() {
    enable_raw_mode().unwrap();
    let mut out = std::io::stdout();
    loop {
        match read() {
            Ok(Event::Key(e)) => {
                if e.kind != KeyEventKind::Press { continue; }
                match e.code {
                    KeyCode::Char(c) => {
                        let size = size().unwrap();
                        let pos = position().unwrap();
                        write!(out, "{} pressed.", c).unwrap();
                        if pos.1 == size.1-1 {
                            execute!(out, ScrollUp(1), MoveToColumn(0)).unwrap();
                        } else {
                            execute!(out, MoveToNextLine(1)).unwrap();
                        }
                    },
                    KeyCode::Esc => return,
                    _ => (),
                }
            },
            _ => (),
        }
    }
}
