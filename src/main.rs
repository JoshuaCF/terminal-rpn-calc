mod command;
mod application;

use application::{Calculator, Response};
use command::Command;

use crossterm::execute;
use crossterm::event::read;
use crossterm::terminal::{enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};


fn main() -> Result<(), std::io::Error> {
    enable_raw_mode().unwrap();

    let mut app = Calculator::new(Box::new(std::io::stdout()));
    execute!(std::io::stdout(), EnterAlternateScreen, Clear(ClearType::All))?;
    loop {
        app.process_command(Command::Draw)?;
        let code = app.process_event(read().unwrap())?;
        match code {
            Response::NoAction => (),
            Response::Exit => break,
        }
    }

    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
