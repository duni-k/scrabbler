use crate::direction::Direction;

use cursive::event::{Event, Key};

pub enum ScrabbleEvent {
    Move(Direction),
    Letter(char),
    Confirm,
    Delete,
    Undo,
    Redo,
    Ignored,
}

impl From<Event> for ScrabbleEvent {
    fn from(event: Event) -> Self {
        match event {
            Event::Key(Key::Up) | Event::Char('k') => Self::Move(Direction::Up),
            Event::Key(Key::Down) | Event::Char('j') => Self::Move(Direction::Down),
            Event::Key(Key::Left) | Event::Char('h') => Self::Move(Direction::Left),
            Event::Key(Key::Right) | Event::Char('l') => Self::Move(Direction::Right),
            Event::Key(Key::Del | Key::Backspace) => Self::Delete,
            Event::Char(ch @ ('a'..='z' | 'å'..='ö' | 'A'..='Z' | 'Å'..='Ö')) => {
                Self::Letter(ch)
            }
            Event::CtrlChar('z') => Self::Undo,
            Event::CtrlChar('r') => Self::Redo,
            Event::Key(Key::Enter) => Self::Confirm,
            _ => Self::Ignored,
        }
    }
}
