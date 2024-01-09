use crate::direction::Direction;

use cursive::{theme::ColorStyle, view::View, Printer, Vec2};
use std::{collections::HashMap, fmt};

const BOARD_SIZE: usize = 15;

#[derive(Default)]
pub struct ScrabbleBoard {
    squares: Vec<Square>,
    focus: Vec2,
    pub tentative: HashMap<Vec2, char>,
    pub size: Vec2,
}

impl ScrabbleBoard {
    pub fn new() -> Self {
        let mut board = Self {
            squares: vec![Square::default(); BOARD_SIZE * BOARD_SIZE],
            focus: Vec2::both_from((BOARD_SIZE - 1) / 2),
            size: Vec2::both_from(BOARD_SIZE),
            tentative: HashMap::new(),
        };
        board.initialize_multipliers();
        board
    }

    pub fn move_focus(&mut self, dir: &Direction) {
        self.focus = match dir {
            Direction::Down => self.focus.map_y(|y| y + 1),
            Direction::Up => self.focus.map_y(|y| if y > 0 { y } else { BOARD_SIZE } - 1),
            Direction::Right => self.focus.map_x(|x| x + 1),
            Direction::Left => self.focus.map_x(|x| if x > 0 { x } else { BOARD_SIZE } - 1),
            Direction::Unknown => self.focus,
        }
        .map(|v| v % BOARD_SIZE);
    }

    pub fn place_focused(&mut self, letter: char) {
        self.squares[Self::coords_to_index(self.focus.x, self.focus.y)].ch = Some(letter);
    }

    pub fn place_at(&mut self, letter: char, pos: &Vec2) {
        self.squares[Self::coords_to_index(pos.x, pos.y)].ch = Some(letter);
    }

    pub fn place_tentative(&mut self) {
        for (pos, letter) in &self.tentative {
            self.squares[Self::coords_to_index(pos.x, pos.y)].ch = Some(letter.clone());
        }
        self.tentative.clear();
    }

    pub fn focus(&self) -> &Vec2 {
        &self.focus
    }

    pub fn clear_focused(&mut self) -> Option<char> {
        self.focused_square_mut().clear_char()
    }

    pub fn focused_char(&self) -> Option<char> {
        self.focused_square().ch
    }

    fn focused_square(&self) -> &Square {
        &self.squares[Self::coords_to_index(self.focus.x, self.focus.y)]
    }

    fn focused_square_mut(&mut self) -> &mut Square {
        self.squares
            .get_mut(Self::coords_to_index(self.focus.x, self.focus.y))
            .unwrap()
    }

    pub fn letter_at(&self, pos: &Vec2) -> Option<char> {
        self.squares
            .get(Self::coords_to_index(pos.x, pos.y))
            .and_then(|square| square.ch)
    }

    fn square_mut_unchecked(&mut self, pos: &Vec2) -> &mut Square {
        self.squares
            .get_mut(Self::coords_to_index(pos.x, pos.y))
            .unwrap()
    }

    fn square_from_coords_unchecked(&self, x: usize, y: usize) -> &Square {
        self.squares.get(Self::coords_to_index(x, y)).unwrap()
    }

    pub fn neighbors(&self, pos: &Vec2) -> Vec<Vec2> {
        let neighbors = vec![
            (pos.x - 1, pos.y),
            (pos.x + 1, pos.y),
            (pos.x, pos.y + 1),
            (pos.x, pos.y - 1),
        ];

        neighbors
            .iter()
            .filter_map(|p| {
                if self.squares[Self::coords_to_index(p.0, p.1)].ch.is_some() {
                    Some(Vec2::new(p.0, p.1))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn has_letter_unchecked(&self, x: usize, y: usize) -> bool {
        self.square_from_coords_unchecked(x, y).ch.is_some()
    }

    fn square_mut_from_coords_unchecked(&mut self, x: usize, y: usize) -> &mut Square {
        self.squares.get_mut(Self::coords_to_index(x, y)).unwrap()
    }

    fn initialize_multipliers(&mut self) {
        const HALF_WAY: usize = (BOARD_SIZE - 1) / 2;
        let init_mult = HashMap::from([
            (
                Multiplier::Tw,
                vec![Vec2::zero(), Vec2::new(0, HALF_WAY), Vec2::new(HALF_WAY, 0)],
            ),
            (
                Multiplier::Tl,
                vec![
                    Vec2::new(1, HALF_WAY - 2),
                    Vec2::new(HALF_WAY - 2, 1),
                    Vec2::new(HALF_WAY - 2, HALF_WAY - 2),
                ],
            ),
            (
                Multiplier::Dw,
                (1..5)
                    .into_iter()
                    .map(|n| Vec2::new(n, n))
                    .collect::<Vec<Vec2>>(),
            ),
            (
                Multiplier::Dl,
                vec![
                    Vec2::new(0, 3),
                    Vec2::new(HALF_WAY, 3),
                    Vec2::new(3, 0),
                    Vec2::new(3, HALF_WAY),
                    Vec2::new(2, HALF_WAY - 1),
                    Vec2::new(HALF_WAY - 1, 2),
                    Vec2::new(HALF_WAY - 1, HALF_WAY - 1),
                ],
            ),
        ]);

        for (mult, positions) in &init_mult {
            for pos in positions {
                self.square_mut_unchecked(&pos).mult = Some(mult.clone());
            }
        }

        for y in 0..(HALF_WAY + 1) {
            for x in 0..(HALF_WAY + 1) {
                self.square_mut_from_coords_unchecked(BOARD_SIZE - x - 1, y)
                    .mult = self.square_from_coords_unchecked(x, y).mult;
            }
        }

        for y in 0..(HALF_WAY + 1) {
            for x in 0..(BOARD_SIZE) {
                self.square_mut_from_coords_unchecked(x, BOARD_SIZE - y - 1)
                    .mult = self.square_from_coords_unchecked(x, y).mult;
            }
        }
    }

    fn coords_to_index(x: usize, y: usize) -> usize {
        y * BOARD_SIZE + x
    }
}

impl View for ScrabbleBoard {
    fn draw(&self, printer: &Printer) {
        for (y, row) in self.squares.chunks(BOARD_SIZE).enumerate() {
            for (x, square) in row.iter().enumerate() {
                printer.with_color(
                    if let Some(_) = square.mult && square.ch.is_none() {
                        ColorStyle::merge(ColorStyle::background(), ColorStyle::secondary())
                    } else {
                        ColorStyle::primary()
                    },
                    |printer| {
                        printer.print((4 * x, y), &format!("{}", square));
                    },
                );
            }
        }

        for (pos, ch) in &self.tentative {
            printer.with_color(ColorStyle::secondary(), |printer| {
                printer.print((4 * pos.x, pos.y), &format!("[ {}]", *ch))
            });
        }

        printer.with_color(ColorStyle::highlight(), |printer| {
            let (x, y) = self.focus.pair();
            if let Some(ch) = self.tentative.get(self.focus()) {
                printer.print((4 * x, y), &format!("[ {}]", ch));
            } else {
                printer.print(
                    (4 * x, y),
                    &format!("{}", &self.squares[Self::coords_to_index(x, y)]),
                );
            }
        })
    }

    fn required_size(&mut self, _: Vec2) -> Vec2 {
        self.size.map_x(|x| x * 4)
    }
}

#[derive(Copy, Clone)]
struct Square {
    pub ch: Option<char>,
    mult: Option<Multiplier>,
}

impl Square {
    fn clear_char(&mut self) -> Option<char> {
        let ch = self.ch;
        self.ch = None;
        ch
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{}]",
            if let Some(ch) = self.ch {
                String::from(ch) + " "
            } else if let Some(mult) = self.mult {
                mult.to_string()
            } else {
                String::from("  ")
            }
        )
    }
}

impl Default for Square {
    fn default() -> Self {
        Self {
            ch: None,
            mult: None,
        }
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub enum Multiplier {
    Tw,
    Dw,
    Tl,
    Dl,
}

impl Multiplier {
    fn as_factor(&self) -> usize {
        match self {
            Self::Dw | Self::Dl => 2,
            Self::Tw | Self::Tl => 3,
        }
    }
}

impl fmt::Display for Multiplier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Tw => "TW",
                Self::Dw => "DW",
                Self::Tl => "TL",
                Self::Dl => "DL",
            }
        )
    }
}
