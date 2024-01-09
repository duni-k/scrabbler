#[derive(Default, Clone, Copy)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
    #[default]
    Unknown,
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Unknown => Self::Unknown,
        }
    }
}
