use std::{
    collections::{HashMap, HashSet},
    fmt,
};

use cursive::{theme::ColorStyle, view::View, Printer, Vec2};
use itertools::Itertools;

use crate::direction::Direction;

const BOARD_SIZE: usize = 15;

#[derive(Default)]
pub struct ScrabbleBoard {
    squares: Vec<Square>,
    focus: Vec2,
    pub tentative: HashSet<Vec2>,
    pub size: Vec2,
    inserted: usize,
}

impl ScrabbleBoard {
    pub fn new() -> Self {
        let mut board = Self {
            squares: vec![Square::default(); BOARD_SIZE * BOARD_SIZE],
            focus: Vec2::both_from((BOARD_SIZE - 1) / 2),
            size: Vec2::both_from(BOARD_SIZE),
            tentative: HashSet::new(),
            inserted: 0,
        };
        board.initialize_multipliers();
        board
    }

    // BFS through the board to make sure it's all connected
    pub fn is_connected(&self) -> bool {
        let mut visited = HashSet::new();
        'outer: for (i, cell) in self.squares.iter().enumerate() {
            if cell.ch.is_some() {
                let mut queue = Vec::new();
                queue.push(self.index_to_coords(i));
                loop {
                    if let Some((x, y)) = queue.pop() {
                        visited.insert((x, y));
                        let mut push_neighbor = |x_n: isize, y_n: isize| {
                            if x_n < (self.size.x as isize)
                                && x_n >= 0
                                && y_n < (self.size.y as isize)
                                && y_n >= 0
                                && !visited.contains(&(x_n as usize, y_n as usize))
                                && self
                                    .letter_at(&Vec2::new(x_n as usize, y_n as usize))
                                    .is_some()
                            {
                                queue.push((x_n as usize, y_n as usize));
                            }
                        };
                        push_neighbor(x as isize + 1, y as isize);
                        push_neighbor(x as isize - 1, y as isize);
                        push_neighbor(x as isize, y as isize + 1);
                        push_neighbor(x as isize, y as isize - 1);
                    } else {
                        break 'outer;
                    }
                }
            }
        }

        visited.len() == self.inserted
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

    pub fn place_focused(&mut self, letter: char) -> Option<char> {
        let focus = self.focus().clone();
        self.place_at(letter, &focus)
    }

    pub fn place_at(&mut self, letter: char, pos: &Vec2) -> Option<char> {
        let existing_ch = self.squares[Self::coords_to_index(pos.x, pos.y)].ch;
        if existing_ch.is_none() {
            self.inserted += 1;
        }
        self.squares[Self::coords_to_index(pos.x, pos.y)].ch = Some(letter);
        existing_ch
    }

    pub fn focus(&self) -> &Vec2 {
        &self.focus
    }

    pub fn clear_focused(&mut self) -> Option<char> {
        self.inserted -= 1;
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

    pub fn square_at(&self, pos: &Vec2) -> Option<&Square> {
        self.squares.get(Self::coords_to_index(pos.x, pos.y))
    }

    fn square_mut_unchecked(&mut self, pos: &Vec2) -> &mut Square {
        self.squares
            .get_mut(Self::coords_to_index(pos.x, pos.y))
            .unwrap()
    }

    pub fn center_square(&self) -> Vec2 {
        self.size.map(|v| (v - 1) / 2)
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

    pub fn mult_at(&self, x: usize, y: usize) -> Option<Multiplier> {
        self.square_from_coords_unchecked(x, y).mult
    }

    pub fn clear_tentative(&mut self) -> Vec<char> {
        let mut cleared = Vec::new();
        for pos in self.tentative.clone() {
            cleared.push(self.square_mut_unchecked(&pos).clear_char().unwrap());
        }
        self.inserted -= self.tentative.len();
        self.tentative.clear();
        cleared
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

    pub fn tentative_alignment(&self) -> Alignment {
        match self.tentative.len() {
            0 => return Alignment::Invalid,
            1 => return Alignment::Undefined,
            2 => {
                let mut tent = self.tentative.iter();
                Alignment::new(tent.next().unwrap(), tent.next().unwrap())
            }
            _ => {
                let mut a = None;
                for (this, next) in self.tentative.iter().tuple_windows() {
                    if a.is_none() {
                        a = Some(Alignment::new(&this, &next));
                    } else if a != Some(Alignment::new(&this, &next)) {
                        return Alignment::Invalid;
                    }
                }
                a.unwrap()
            }
        }
    }

    pub fn collect_tentative(&self) -> Result<Vec<Vec<Square>>, String> {
        let horizontal_pred = |pos: Vec2| pos.map_x(|x| x - 1);
        let horizontal_succ = |pos: Vec2| pos.map_x(|x| x + 1);
        let vertical_pred = |pos: Vec2| pos.map_y(|y| y - 1);
        let vertical_succ = |pos: Vec2| pos.map_y(|y| y + 1);

        match self.tentative_alignment() {
            Alignment::Horizontal => Ok(self.collecter_aux(
                horizontal_pred,
                horizontal_succ,
                vertical_pred,
                vertical_succ,
            )),
            Alignment::Vertical => Ok(self.collecter_aux(
                vertical_pred,
                vertical_succ,
                horizontal_pred,
                horizontal_succ,
            )),
            Alignment::Undefined => {
                let mut curr = *self.tentative.iter().next().unwrap();
                let mut hori = Vec::new();
                loop {
                    let prev = horizontal_pred(curr);
                    if let Some(_) = self.letter_at(&prev) {
                        curr = prev;
                    } else {
                        break;
                    }
                }
                while let Some(square) = self.square_at(&curr) {
                    if square.ch.is_none() {
                        break;
                    }
                    hori.push(square.clone());
                    curr = horizontal_succ(curr);
                }

                let mut curr = *self.tentative.iter().next().unwrap();
                let mut vert = Vec::new();
                loop {
                    let prev = vertical_pred(curr);
                    if let Some(_) = self.letter_at(&prev) {
                        curr = prev;
                    } else {
                        break;
                    }
                }
                while let Some(square) = self.square_at(&curr) {
                    if square.ch.is_none() {
                        break;
                    }
                    vert.push(square.clone());
                    curr = vertical_succ(curr);
                }
                match (hori.len(), vert.len()) {
                    (_, 1) => Ok(vec![hori]),
                    (1, _) => Ok(vec![vert]),
                    (_, _) => Ok(vec![hori, vert]),
                }
            }
            Alignment::Invalid => return Err("Letters not aligned".to_string()),
        }
    }

    fn collecter_aux(
        &self,
        outer_pred: impl Fn(Vec2) -> Vec2,
        outer_succ: impl Fn(Vec2) -> Vec2,
        inner_pred: impl Fn(Vec2) -> Vec2,
        inner_succ: impl Fn(Vec2) -> Vec2,
    ) -> Vec<Vec<Square>> {
        let mut word_squares: Vec<Vec<Square>> = Vec::new();

        let mut curr_main = *self.tentative.iter().next().unwrap();
        loop {
            let prev = outer_pred(curr_main);
            if let Some(_) = self.letter_at(&prev) {
                curr_main = prev;
            } else {
                break;
            }
        }

        let mut main_squares: Vec<Square> = Vec::new();
        while let Some(square) = self.square_at(&curr_main) {
            let mut inner_squares: Vec<Square> = Vec::new();
            if square.ch.is_none() {
                break;
            }
            main_squares.push(square.clone());
            if self.tentative.contains(&curr_main) {
                let mut curr = curr_main.clone();
                match (
                    self.letter_at(&inner_pred(curr_main)),
                    self.letter_at(&inner_succ(curr_main)),
                ) {
                    (None, None) | (Some(_), Some(_)) => (),
                    (Some(_), None) => {
                        inner_squares.push(square.clone());
                        loop {
                            let next = inner_pred(curr);
                            if let Some(square) = self.square_at(&next) {
                                if square.ch.is_none() {
                                    break;
                                }
                                inner_squares.insert(0, square.clone());
                                curr = next;
                            } else {
                                break;
                            }
                        }
                        word_squares.push(inner_squares);
                    }
                    (None, Some(_)) => {
                        inner_squares.push(square.clone());
                        loop {
                            let next = inner_succ(curr);
                            if let Some(square) = self.square_at(&next) {
                                if square.ch.is_none() {
                                    break;
                                }
                                inner_squares.push(square.clone());
                                curr = next;
                            } else {
                                break;
                            }
                        }
                        word_squares.push(inner_squares);
                    }
                }
            }
            curr_main = outer_succ(curr_main);
        }
        word_squares.push(main_squares);

        word_squares
    }

    fn index_to_coords(&self, idx: usize) -> (usize, usize) {
        (idx % self.size.x, idx / self.size.y)
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
                    if square.mult.is_none() || square.ch.is_some() {
                        ColorStyle::primary()
                    } else {
                        ColorStyle::merge(ColorStyle::background(), ColorStyle::primary())
                    },
                    |printer| {
                        printer.print((4 * x, y), &format!("{}", square));
                    },
                );
            }
        }

        for pos in &self.tentative {
            printer.with_color(ColorStyle::secondary(), |printer| {
                printer.print(
                    (4 * pos.x, pos.y),
                    &format!("[{} ]", self.letter_at(pos).unwrap()),
                )
            });
        }

        printer.with_color(ColorStyle::highlight(), |printer| {
            let (x, y) = self.focus.pair();
            if let Some(ch) = self.letter_at(self.focus()) {
                printer.print((4 * x, y), &format!("[{} ]", ch));
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
pub struct Square {
    pub ch: Option<char>,
    pub mult: Option<Multiplier>,
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
    pub fn as_factor(&self) -> usize {
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

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Alignment {
    Horizontal,
    Vertical,
    Undefined,
    Invalid,
}

impl Alignment {
    fn new(a: &Vec2, b: &Vec2) -> Self {
        if a.x != b.x && a.y != b.y {
            Self::Invalid
        } else if a.x == b.x {
            Self::Vertical
        } else {
            Self::Horizontal
        }
    }
}
