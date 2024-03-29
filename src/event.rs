use crate::board::Direction;

use cursive::event::{Event, Key};

pub enum SEvent {
    Move(Direction),
    Letter(char),
    Pass,
    Confirm,
    Shuffle,
    Exchange,
    Delete,
    DeleteAll,
    Ignored,
}

impl From<Event> for SEvent {
    fn from(event: Event) -> Self {
        match event {
            Event::Key(Key::Up) | Event::Char('K') => Self::Move(Direction::Up),
            Event::Key(Key::Down) | Event::Char('J') => Self::Move(Direction::Down),
            Event::Key(Key::Left) | Event::Char('H') => Self::Move(Direction::Left),
            Event::Key(Key::Right) | Event::Char('L') => Self::Move(Direction::Right),
            Event::Key(Key::Del | Key::Backspace) => Self::Delete,
            Event::Char(ch @ ('a'..='z' | 'å'..='ö')) => Self::Letter(ch),
            Event::CtrlChar('p') => Self::Pass,
            Event::CtrlChar('e') => Self::Exchange,
            Event::CtrlChar('d') => Self::DeleteAll,
            Event::CtrlChar('r') => Self::Shuffle,
            Event::Key(Key::Enter) => Self::Confirm,
            _ => Self::Ignored,
        }
    }
}
