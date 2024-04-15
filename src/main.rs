use crossterm::*;
use crossterm::cursor::*;
use crossterm::event::*;
use crossterm::terminal::*;

use std::io::{stdout, Write};

enum Command {
    AppendToBfr(char),
    Enter,
    BinOp(BinOp),
    Exit,
    NoOp,
}
enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

fn process_char(kchar: KeyEvent) -> Result<Command, std::io::Error> {
    if kchar.modifiers.bits() & !KeyModifiers::SHIFT.bits() != 0 { return Ok(Command::NoOp); }
    let c = match kchar.code {
        KeyCode::Char(c) => c,
        _ => panic!(),
    };
    
    match c {
        '+' => Ok(Command::BinOp(BinOp::Add)),
        '-' => Ok(Command::BinOp(BinOp::Sub)),
        '*' => Ok(Command::BinOp(BinOp::Mul)),
        '/' => Ok(Command::BinOp(BinOp::Div)),
        '0'..='9' | '.' => Ok(Command::AppendToBfr(c)),
        _ => Ok(Command::NoOp),
    }
}
// fn process_arrow(ar: KeyCode) -> Result<(), std::io::Error> {
//     match ar {
//         KeyCode::Up => execute!(stdout(), MoveUp(1))?,
//         KeyCode::Right => execute!(stdout(), MoveRight(1))?,
//         KeyCode::Down => execute!(stdout(), MoveDown(1))?,
//         KeyCode::Left => execute!(stdout(), MoveLeft(1))?,
//         _ => (),
//     }
//     Ok(())
// }

const STACK_SIZE: usize = 12;
struct NumStack {
    nums: [f64; STACK_SIZE],
}
impl NumStack {
    fn new() -> NumStack {
        NumStack { nums: [0.0; STACK_SIZE] }
    }
    fn rotate_in(&mut self, num: f64) {
        for i in (0..STACK_SIZE-1).rev() {
            self.nums[i+1] = self.nums[i];
        }
        self.nums[0] = num;
    }
    fn rotate_out(&mut self, num: f64) {
        for i in 1..STACK_SIZE {
            self.nums[i-1] = self.nums[i];
        }
        self.nums[0] = num;
    }

    fn add(&mut self) {
        let res = self.nums[1] + self.nums[0];
        self.rotate_out(res);
    }
    fn sub(&mut self) {
        let res = self.nums[1] - self.nums[0];
        self.rotate_out(res);
    }
    fn mul(&mut self) {
        let res = self.nums[1] * self.nums[0];
        self.rotate_out(res);
    }
    fn div(&mut self) {
        let res = self.nums[1] / self.nums[0];
        self.rotate_out(res);
    }
}

fn main() -> Result<(), std::io::Error> {
    enable_raw_mode().unwrap();

    let mut stack = NumStack::new();
    let mut in_bfr = String::with_capacity(256);
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, Clear(ClearType::All))?;
    loop {
        queue!(out, MoveTo(0, 0))?;
        for v in stack.nums.iter().rev() {
            write!(out, "{:.>12.4}", v)?;
            queue!(out, Clear(ClearType::UntilNewLine), MoveToNextLine(1))?;
        }
        queue!(out, Clear(ClearType::UntilNewLine), MoveToNextLine(1))?;
        write!(out, "{}", in_bfr)?;
        queue!(out, Clear(ClearType::UntilNewLine))?;
        out.flush()?;

        let e = read()?;
        let mut cmd = Command::NoOp;
        if let Event::Key(ke) = e {
            if ke.kind != KeyEventKind::Press { continue; }
            cmd = match &ke.code {
                KeyCode::Esc => Command::Exit,
                KeyCode::Char(_) => process_char(ke)?,
                KeyCode::Enter => Command::Enter,
                _ => continue,
            };
        }

        match cmd {
            Command::AppendToBfr(c) => in_bfr.push(c),
            Command::Enter => {
                match in_bfr.parse::<f64>() {
                    Ok(v) => stack.rotate_in(v),
                    Err(_) => {
                        if in_bfr.is_empty() {
                            stack.rotate_in(stack.nums[0]);
                        }
                    },
                };
                in_bfr.clear();
            },
            Command::BinOp(op) => {
                if !in_bfr.is_empty() {
                    match in_bfr.parse::<f64>() {
                        Ok(v) => stack.rotate_in(v),
                        Err(_) => (),
                    };
                    in_bfr.clear();
                }
                match op {
                    BinOp::Add => stack.add(),
                    BinOp::Sub => stack.sub(),
                    BinOp::Mul => stack.mul(),
                    BinOp::Div => stack.div(),
                }
                in_bfr.clear();
            },
            Command::Exit => break,
            Command::NoOp => (),
        }
    }

    execute!(out, LeaveAlternateScreen)?;
    Ok(())
}
