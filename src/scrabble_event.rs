use crate::direction::Direction;

use cursive::event::{Event, Key};

pub enum ScrabbleEvent {
    Move(Direction),
    Letter(char),
    Pass,
    Confirm,
    Delete,
    Undo,
    Redo,
    Ignored,
}

impl From<Event> for ScrabbleEvent {
    fn from(event: Event) -> Self {
        match event {
            Event::Key(Key::Up) | Event::Char('K') => Self::Move(Direction::Up),
            Event::Key(Key::Down) | Event::Char('J') => Self::Move(Direction::Down),
            Event::Key(Key::Left) | Event::Char('H') => Self::Move(Direction::Left),
            Event::Key(Key::Right) | Event::Char('L') => Self::Move(Direction::Right),
            Event::Key(Key::Del | Key::Backspace) => Self::Delete,
            Event::Char(ch @ ('a'..='z' | 'å'..='ö')) => Self::Letter(ch),
            Event::CtrlChar('z') => Self::Undo,
            Event::CtrlChar('r') => Self::Redo,
            Event::CtrlChar('p') => Self::Pass,
            Event::Key(Key::Enter) => Self::Confirm,
            _ => Self::Ignored,
        }
    }
}
