use std::{
    collections::{HashMap, HashSet},
    fmt,
};

use cursive::{
    theme::{BaseColor::*, ColorStyle},
    view::View,
    Printer, Vec2,
};
use itertools::Itertools;

const BOARD_SIZE: usize = 15;

#[derive(Clone)]
pub struct Board {
    focus: Vec2,
    inserted: HashSet<Vec2>,
    pub size: Vec2,
    pub tentative: HashSet<Vec2>,
    cells: Vec<Cell>,
}

#[derive(Clone)]
pub struct Cell {
    pub ch: Option<char>,
    pub mult: Option<Multiplier>,
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub enum Multiplier {
    Tw,
    Dw,
    Tl,
    Dl,
}

/// Represents the alignment that the placement of tiles on the board corresponds with.
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Alignment {
    Horizontal,
    Vertical,
    Invalid,
}

pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Board {
    pub fn new() -> Self {
        let mut board = Self {
            cells: vec![Cell::default(); BOARD_SIZE * BOARD_SIZE],
            focus: Vec2::both_from((BOARD_SIZE - 1) / 2),
            size: Vec2::both_from(BOARD_SIZE),
            tentative: HashSet::new(),
            inserted: HashSet::new(),
        };
        board.initialize_multipliers();
        board
    }

    pub fn inserted(&self) -> &HashSet<Vec2> {
        &self.inserted
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
                        let mut push_neighbor = |x_n, y_n| {
                            if self.within_bounds(x as isize, y as isize)
                                && !visited.contains(&(x_n, y_n))
                                && self.letter_at(&Vec2::new(x_n, y_n)).is_some()
                            {
                                queue.push((x_n, y_n));
                            }
                        };
                        push_neighbor(x + 1, y);
                        push_neighbor(x - 1, y);
                        push_neighbor(x, y + 1);
                        push_neighbor(x, y - 1);
                    } else {
                        break 'outer;
                    }
                }
            }
        }

        visited.len() == self.inserted.len()
    }

    pub fn within_bounds(&self, x: isize, y: isize) -> bool {
        x < (self.size.x as isize) && x >= 0 && y < (self.size.y as isize) && y >= 0
    }

    pub fn move_focus(&mut self, dir: &Direction) {
        self.focus = match dir {
            Direction::Down => self.focus.map_y(|y| y + 1),
            Direction::Up => self.focus.map_y(|y| if y > 0 { y } else { BOARD_SIZE } - 1),
            Direction::Right => self.focus.map_x(|x| x + 1),
            Direction::Left => self.focus.map_x(|x| if x > 0 { x } else { BOARD_SIZE } - 1),
        }
        .map(|v| v % BOARD_SIZE);
    }

    pub fn place_focused(&mut self, letter: char) -> Option<char> {
        let focus = self.focus().clone();
        self.place_at(letter, &focus)
    }

    pub fn place_at(&mut self, letter: char, pos: &Vec2) -> Option<char> {
        let existing_ch = self.cells[Self::coords_to_index(pos.x, pos.y)].ch;
        self.inserted.insert(pos.clone());
        self.cells[Self::coords_to_index(pos.x, pos.y)].ch = Some(letter);
        existing_ch
    }

    pub fn focus(&self) -> &Vec2 {
        &self.focus
    }

    pub fn clear_focused(&mut self) -> Option<char> {
        self.inserted.remove(&self.focus);
        self.focused_cell_mut().clear_letter()
    }

    pub fn focused_letter(&self) -> Option<char> {
        self.focused_cell().ch
    }

    fn focused_cell(&self) -> &Cell {
        &self.cells[Self::coords_to_index(self.focus.x, self.focus.y)]
    }

    fn focused_cell_mut(&mut self) -> &mut Cell {
        self.cells
            .get_mut(Self::coords_to_index(self.focus.x, self.focus.y))
            .unwrap()
    }

    pub fn letter_at(&self, pos: &Vec2) -> Option<char> {
        self.cells
            .get(Self::coords_to_index(pos.x, pos.y))
            .and_then(|cell| cell.ch)
    }

    pub fn letter_at_coords(&self, x: usize, y: usize) -> Option<char> {
        self.cells
            .get(Self::coords_to_index(x, y))
            .and_then(|cell| cell.ch)
    }

    pub fn cell_at(&self, pos: &Vec2) -> Option<&Cell> {
        self.cells.get(Self::coords_to_index(pos.x, pos.y))
    }

    fn cell_mut(&mut self, pos: &Vec2) -> Option<&mut Cell> {
        self.cells.get_mut(Self::coords_to_index(pos.x, pos.y))
    }

    pub fn center_pos(&self) -> Vec2 {
        self.size.map(|v| (v - 1) / 2)
    }

    fn cell_from_coords(&self, x: usize, y: usize) -> Option<&Cell> {
        self.cells.get(Self::coords_to_index(x, y))
    }

    pub fn vacant_neighbors(&self, pos: &Vec2) -> Vec<Vec2> {
        let neighbors = vec![
            pos.map_x(|x| x - 1),
            pos.map_x(|x| x + 1),
            pos.map_y(|y| y + 1),
            pos.map_y(|y| y - 1),
        ];

        neighbors
            .iter()
            .filter_map(|&p| {
                if self.cells[Self::coords_to_index(p.x, p.y)].ch.is_none() {
                    Some(p)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn has_letter(&self, x: usize, y: usize) -> bool {
        self.cell_from_coords(x, y)
            .map_or(false, |cell| cell.ch.is_some())
    }

    pub fn mult_at(&self, x: usize, y: usize) -> Option<Multiplier> {
        self.cell_from_coords(x, y).and_then(|cell| cell.mult)
    }

    pub fn clear_tentative_from_board(&mut self) -> Vec<char> {
        let mut cleared = Vec::new();
        for pos in self.tentative.clone() {
            cleared.push(self.cell_mut(&pos).unwrap().clear_letter().unwrap());
            self.tentative.remove(&pos);
        }
        self.tentative.clear();
        cleared
    }

    fn cell_mut_from_coords(&mut self, x: usize, y: usize) -> Option<&mut Cell> {
        self.cells.get_mut(Self::coords_to_index(x, y))
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
                self.cell_mut(&pos).unwrap().mult = Some(mult.clone());
            }
        }

        for y in 0..(HALF_WAY + 1) {
            for x in 0..(HALF_WAY + 1) {
                self.cell_mut_from_coords(BOARD_SIZE - x - 1, y)
                    .unwrap()
                    .mult = self.cell_from_coords(x, y).unwrap().mult;
            }
        }

        for y in 0..(HALF_WAY + 1) {
            for x in 0..(BOARD_SIZE) {
                self.cell_mut_from_coords(x, BOARD_SIZE - y - 1)
                    .unwrap()
                    .mult = self.cell_from_coords(x, y).unwrap().mult;
            }
        }
    }

    pub fn tentative_alignment(&self) -> Option<Alignment> {
        match self.tentative.len() {
            0 => return Some(Alignment::Invalid),
            1 => return None,
            2 => {
                let mut tent = self.tentative.iter();
                Some(Alignment::new(tent.next().unwrap(), tent.next().unwrap()))
            }
            _ => {
                let mut a = None;
                for (this, next) in self.tentative.iter().tuple_windows() {
                    if a.is_none() {
                        a = Some(Alignment::new(&this, &next));
                    } else if a != Some(Alignment::new(&this, &next)) {
                        return Some(Alignment::Invalid);
                    }
                }
                a
            }
        }
    }

    pub fn collect_tentative(&mut self) -> Result<Vec<Vec<Cell>>, String> {
        let horizontal_pred = |pos: Vec2| pos.map_x(|x| x - 1);
        let horizontal_succ = |pos: Vec2| pos.map_x(|x| x + 1);
        let vertical_pred = |pos: Vec2| pos.map_y(|y| y - 1);
        let vertical_succ = |pos: Vec2| pos.map_y(|y| y + 1);

        let mut mults_to_clear: Vec<Vec2> = Vec::new();
        let res = match self.tentative_alignment() {
            Some(Alignment::Horizontal) => Ok(self.collecter_aux(
                &mut mults_to_clear,
                horizontal_pred,
                horizontal_succ,
                vertical_pred,
                vertical_succ,
            )),
            Some(Alignment::Vertical) => Ok(self.collecter_aux(
                &mut mults_to_clear,
                vertical_pred,
                vertical_succ,
                horizontal_pred,
                horizontal_succ,
            )),
            None => {
                let mut curr = *self.tentative.iter().next().unwrap();
                let mut mults_to_clear_hori = Vec::new();
                while let Some(_) = self.letter_at(&horizontal_pred(curr)) {
                    curr = horizontal_pred(curr);
                }
                let mut hori = Vec::new();
                while let Some(cell) = self.cell_at(&curr) {
                    if cell.ch.is_none() {
                        break;
                    }
                    hori.push(cell.clone());
                    mults_to_clear_hori.push(curr.clone());
                    curr = horizontal_succ(curr);
                }

                let mut curr = *self.tentative.iter().next().unwrap();
                while let Some(_) = self.letter_at(&vertical_pred(curr)) {
                    curr = vertical_pred(curr);
                }

                let mut vert = Vec::new();
                while let Some(cell) = self.cell_at(&curr) {
                    if cell.ch.is_none() {
                        break;
                    }
                    vert.push(cell.clone());
                    mults_to_clear.push(curr.clone());
                    curr = vertical_succ(curr);
                }
                match (hori.len(), vert.len()) {
                    (_, 1) => {
                        mults_to_clear = mults_to_clear_hori;
                        Ok(vec![hori])
                    }
                    (1, _) => Ok(vec![vert]),
                    (_, _) => {
                        mults_to_clear.append(&mut mults_to_clear_hori);
                        Ok(vec![hori, vert])
                    }
                }
            }
            Some(Alignment::Invalid) => return Err("Letters not aligned".to_string()),
        };

        if res.is_ok() {
            for pos in mults_to_clear {
                self.cell_mut_from_coords(pos.x, pos.y).unwrap().mult = None;
            }
        }

        res
    }

    fn collecter_aux(
        &self,
        mults_to_clear: &mut Vec<Vec2>,
        outer_pred: impl Fn(Vec2) -> Vec2,
        outer_succ: impl Fn(Vec2) -> Vec2,
        inner_pred: impl Fn(Vec2) -> Vec2,
        inner_succ: impl Fn(Vec2) -> Vec2,
    ) -> Vec<Vec<Cell>> {
        let mut word_cells: Vec<Vec<Cell>> = Vec::new();

        let mut curr_main = *self.tentative.iter().next().unwrap();
        while let Some(_) = self.letter_at(&outer_pred(curr_main)) {
            curr_main = outer_pred(curr_main);
        }

        let mut main_cells: Vec<Cell> = Vec::new();
        while let Some(cell) = self.cell_at(&curr_main) {
            let mut inner_cells: Vec<Cell> = Vec::new();
            if cell.ch.is_none() {
                break;
            }
            main_cells.push(cell.clone());
            mults_to_clear.push(curr_main.clone());
            if self.tentative.contains(&curr_main) {
                let mut curr = curr_main.clone();
                match (
                    self.letter_at(&inner_pred(curr_main)),
                    self.letter_at(&inner_succ(curr_main)),
                ) {
                    (None, None) | (Some(_), Some(_)) => (),
                    (Some(_), None) => {
                        while let Some(cell) = self.cell_at(&curr) {
                            if cell.ch.is_none() {
                                break;
                            }
                            inner_cells.insert(0, cell.clone());
                            mults_to_clear.insert(0, curr.clone());
                            curr = inner_pred(curr);
                        }
                        word_cells.push(inner_cells);
                    }
                    (None, Some(_)) => {
                        while let Some(cell) = self.cell_at(&curr) {
                            if cell.ch.is_none() {
                                break;
                            }
                            inner_cells.push(cell.clone());
                            mults_to_clear.push(curr.clone());
                            curr = inner_succ(curr);
                        }
                        word_cells.push(inner_cells);
                    }
                }
            }
            curr_main = outer_succ(curr_main);
        }
        word_cells.push(main_cells);

        word_cells
    }

    pub fn index_to_coords(&self, idx: usize) -> (usize, usize) {
        (idx % self.size.x, idx / self.size.y)
    }

    pub fn coords_to_index(x: usize, y: usize) -> usize {
        y * BOARD_SIZE + x
    }
}

impl View for Board {
    fn draw(&self, printer: &Printer) {
        for (y, row) in self.cells.chunks(BOARD_SIZE).enumerate() {
            for (x, cell) in row.iter().enumerate() {
                printer.with_color(
                    match cell.mult {
                        _ if cell.ch.is_some() => ColorStyle::primary(),
                        Some(Multiplier::Dl) => ColorStyle::new(Black, Blue),
                        Some(Multiplier::Tl) => ColorStyle::new(Black, Blue.light()),
                        Some(Multiplier::Dw) => ColorStyle::new(Black, Red),
                        Some(Multiplier::Tw) => ColorStyle::new(Black, Red.light()),
                        None => ColorStyle::primary(),
                    },
                    |printer| {
                        printer.print((x * Cell::size(), y), &format!("{}", cell));
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
                    (x * Cell::size(), y),
                    &format!("{}", &self.cells[Self::coords_to_index(x, y)]),
                );
            }
        })
    }

    fn required_size(&mut self, _: Vec2) -> Vec2 {
        self.size.map_x(|x| x * 4)
    }
}

impl Cell {
    pub fn clear_letter(&mut self) -> Option<char> {
        let ch = self.ch;
        self.ch = None;
        ch
    }

    pub fn size() -> usize {
        4
    }
}

impl fmt::Display for Cell {
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

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: None,
            mult: None,
        }
    }
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
