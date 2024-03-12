use std::{
    collections::{HashMap, HashSet},
    fmt, mem,
};

use cursive::{
    theme::{BaseColor::*, ColorStyle},
    view::View,
    Printer, Vec2,
};
use itertools::Itertools;

#[derive(Clone)]
pub struct Board {
    focus: Vec2,
    inserted: HashSet<Vec2>,
    pub size: Vec2,
    tentative: HashSet<Vec2>,
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

/// Represents the alignment that the placement of tiles on the board corresponds to.
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
    pub fn new(size: usize) -> Self {
        let mut board = Self {
            cells: vec![Cell::default(); size * size],
            focus: Vec2::both_from((size - 1) / 2),
            size: Vec2::both_from(size),
            tentative: HashSet::new(),
            inserted: HashSet::new(),
        };
        board.initialize_multipliers(size);
        board
    }

    pub fn inserted(&self) -> &HashSet<Vec2> {
        &self.inserted
    }

    // BFS through the board to make sure it's all connected
    pub fn is_connected(&self) -> bool {
        let Some(&inserted) = self.inserted.iter().next() else {
            return false;
        };

        let mut queue = Vec::new();
        let mut visited = HashSet::new();
        let is_occupied = |p: &&Vec2| self.letter_at(p).is_some();
        queue.push(inserted);
        while let Some(pos) = queue.pop() {
            visited.insert(pos);
            for neighbor in self.neighbors_satisfying_predicate(&pos, is_occupied) {
                if !visited.contains(&neighbor) {
                    queue.push(neighbor);
                }
            }
        }

        visited.len() == self.inserted.len()
    }

    pub fn move_focus(&mut self, dir: &Direction) {
        self.focus = match dir {
            Direction::Down => self.focus.map_y(|y| y + 1),
            Direction::Up => self
                .focus
                .map_y(|y| if y > 0 { y } else { self.size.y } - 1),
            Direction::Right => self.focus.map_x(|x| x + 1),
            Direction::Left => self
                .focus
                .map_x(|x| if x > 0 { x } else { self.size.x } - 1),
        }
        .map(|v| v % self.size.x);
    }

    pub fn place_focused(&mut self, letter: char) -> Option<char> {
        self.place_at(letter, &self.focus().clone())
    }

    pub fn place_at(&mut self, letter: char, pos: &Vec2) -> Option<char> {
        let Some(cell) = self.cell_at_mut(pos) else {
            return None;
        };
        let previous = cell.ch;
        cell.ch = Some(letter);
        self.inserted.insert(self.focus.clone());
        self.tentative.insert(self.focus.clone());
        previous
    }

    pub fn place_focused_tentative(&mut self, letter: char) -> Result<Option<char>, &str> {
        if self.letter_at(self.focus()).is_some() && !self.tentative.contains(self.focus()) {
            return Err("Cell occupied");
        }
        Ok(self.place_focused(letter))
    }

    pub fn tentative(&self) -> &HashSet<Vec2> {
        &self.tentative
    }

    pub fn focus(&self) -> &Vec2 {
        &self.focus
    }

    pub fn clear_focused(&mut self) -> Option<char> {
        self.clear_cell(&self.focus().clone())
    }

    fn clear_cell(&mut self, pos: &Vec2) -> Option<char> {
        self.inserted.remove(pos);
        self.tentative.remove(pos);
        self.cell_at_mut(pos).and_then(|cell| cell.clear_letter())
    }

    pub fn focused_letter(&self) -> Option<char> {
        self.focused_cell().ch
    }

    fn focused_cell(&self) -> &Cell {
        self.cell_at(self.focus()).unwrap() // Always Some
    }

    pub fn letter_at(&self, pos: &Vec2) -> Option<char> {
        self.cell_at(pos).and_then(|cell| cell.ch)
    }

    fn cell_at(&self, pos: &Vec2) -> Option<&Cell> {
        self.cells
            .get(Self::coords_to_index(pos.x, pos.y, self.size.y))
    }

    fn cell_at_mut(&mut self, pos: &Vec2) -> Option<&mut Cell> {
        self.cells
            .get_mut(Self::coords_to_index(pos.x, pos.y, self.size.y))
    }

    fn cell_at_coords(&self, x: usize, y: usize) -> Option<&Cell> {
        self.cells.get(Self::coords_to_index(x, y, self.size.y))
    }

    fn cell_at_coords_mut(&mut self, x: usize, y: usize) -> Option<&mut Cell> {
        self.cells.get_mut(Self::coords_to_index(x, y, self.size.y))
    }

    pub fn center_pos(&self) -> Vec2 {
        self.size.map(|v| (v - 1) / 2)
    }

    pub fn vacant_neighbors(&self, pos: &Vec2) -> Vec<Vec2> {
        let is_vacant = |p: &&Vec2| self.letter_at(p).is_none();
        self.neighbors_satisfying_predicate(pos, is_vacant)
    }

    fn neighbors_satisfying_predicate(
        &self,
        pos: &Vec2,
        predicate: impl FnMut(&&Vec2) -> bool,
    ) -> Vec<Vec2> {
        let neighbors = vec![
            pos.map_x(|x| x - 1),
            pos.map_x(|x| x + 1),
            pos.map_y(|y| y + 1),
            pos.map_y(|y| y - 1),
        ];

        neighbors.iter().filter(predicate).cloned().collect()
    }

    pub fn mult_at(&self, x: usize, y: usize) -> Option<Multiplier> {
        self.cell_at_coords(x, y).and_then(|cell| cell.mult)
    }

    //
    pub fn clear_tentative_from_board(&mut self) -> Vec<char> {
        let mut cleared = Vec::new();
        for pos in self.tentative.clone() {
            cleared.push(self.clear_cell(&pos).unwrap());
        }
        self.tentative.clear();
        cleared
    }

    pub fn clear_tentative(&mut self) {
        self.tentative.clear();
    }

    fn cell_mut_at_coords(&mut self, x: usize, y: usize) -> Option<&mut Cell> {
        self.cells.get_mut(Self::coords_to_index(x, y, self.size.y))
    }

    fn initialize_multipliers(&mut self, size: usize) {
        let half_way = (size - 1) / 2;
        let init_mult = HashMap::from([
            (
                Multiplier::Tw,
                vec![Vec2::zero(), Vec2::new(0, half_way), Vec2::new(half_way, 0)],
            ),
            (
                Multiplier::Tl,
                vec![
                    Vec2::new(1, half_way - 2),
                    Vec2::new(half_way - 2, 1),
                    Vec2::new(half_way - 2, half_way - 2),
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
                    Vec2::new(half_way, 3),
                    Vec2::new(3, 0),
                    Vec2::new(3, half_way),
                    Vec2::new(2, half_way - 1),
                    Vec2::new(half_way - 1, 2),
                    Vec2::new(half_way - 1, half_way - 1),
                ],
            ),
        ]);

        for (mult, positions) in &init_mult {
            for pos in positions {
                self.cell_at_mut(&pos).unwrap().mult = Some(mult.clone());
            }
        }

        for y in 0..(half_way + 1) {
            for x in 0..(half_way + 1) {
                self.cell_at_coords_mut(size - x - 1, y).unwrap().mult =
                    self.cell_at_coords(x, y).unwrap().mult;
            }
        }

        for y in 0..(half_way + 1) {
            for x in 0..(size) {
                self.cell_at_coords_mut(x, size - y - 1).unwrap().mult =
                    self.cell_at_coords_mut(x, y).unwrap().mult;
            }
        }
    }

    pub fn tentative_alignment(&self) -> Option<Alignment> {
        let mut tent = self.tentative.iter();
        match self.tentative.len() {
            0 => return Some(Alignment::Invalid),
            1 => return None,
            2 => Some(Alignment::new(tent.next().unwrap(), tent.next().unwrap())),
            _ => {
                let mut a = None;
                for (this, next) in tent.tuple_windows() {
                    if a.is_none() {
                        a = Some(Alignment::new(this, next));
                    } else if a != Some(Alignment::new(this, next)) {
                        return Some(Alignment::Invalid);
                    }
                }
                a
            }
        }
    }

    pub fn collect_tentative(&mut self) -> Result<Vec<Vec<Cell>>, String> {
        let horizontal_pred = |pos: &Vec2| pos.map_x(|x| x - 1);
        let horizontal_succ = |pos: &Vec2| pos.map_x(|x| x + 1);
        let vertical_pred = |pos: &Vec2| pos.map_y(|y| y - 1);
        let vertical_succ = |pos: &Vec2| pos.map_y(|y| y + 1);

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
                while let Some(_) = self.letter_at(&horizontal_pred(&curr)) {
                    curr = horizontal_pred(&curr);
                }
                let mut hori = Vec::new();
                while let Some(cell) = self.cell_at(&curr) {
                    if cell.ch.is_none() {
                        break;
                    }
                    hori.push(cell.clone());
                    mults_to_clear_hori.push(curr.clone());
                    curr = horizontal_succ(&curr);
                }

                let mut curr = *self.tentative.iter().next().unwrap();
                while let Some(_) = self.letter_at(&vertical_pred(&curr)) {
                    curr = vertical_pred(&curr);
                }

                let mut vert = Vec::new();
                while let Some(cell) = self.cell_at(&curr) {
                    if cell.ch.is_none() {
                        break;
                    }
                    vert.push(cell.clone());
                    mults_to_clear.push(curr.clone());
                    curr = vertical_succ(&curr);
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
                self.cell_mut_at_coords(pos.x, pos.y).unwrap().mult = None;
            }
        }

        res
    }

    fn collecter_aux(
        &self,
        mults_to_clear: &mut Vec<Vec2>,
        outer_pred: impl Fn(&Vec2) -> Vec2,
        outer_succ: impl Fn(&Vec2) -> Vec2,
        inner_pred: impl Fn(&Vec2) -> Vec2,
        inner_succ: impl Fn(&Vec2) -> Vec2,
    ) -> Vec<Vec<Cell>> {
        let mut word_cells: Vec<Vec<Cell>> = Vec::new();

        let mut curr_main = *self.tentative.iter().next().unwrap();
        while let Some(_) = self.letter_at(&outer_pred(&curr_main)) {
            curr_main = outer_pred(&curr_main);
        }

        let mut main_cells: Vec<Cell> = Vec::new();
        while let Some(cell) = self.cell_at(&curr_main) {
            let mut inner_cells: Vec<Cell> = Vec::new();
            if cell.ch.is_none() {
                break;
            }
            main_cells.push(cell.clone());
            mults_to_clear.push(curr_main.clone());
            if self.tentative().contains(&curr_main) {
                let mut curr = curr_main.clone();
                match (
                    self.letter_at(&inner_pred(&curr_main)),
                    self.letter_at(&inner_succ(&curr_main)),
                ) {
                    (None, None) | (Some(_), Some(_)) => (),
                    (Some(_), None) => {
                        while let Some(cell) = self.cell_at(&curr) {
                            if cell.ch.is_none() {
                                break;
                            }
                            inner_cells.insert(0, cell.clone());
                            mults_to_clear.insert(0, curr.clone());
                            curr = inner_pred(&curr);
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
                            curr = inner_succ(&curr);
                        }
                        word_cells.push(inner_cells);
                    }
                }
            }
            curr_main = outer_succ(&curr_main);
        }
        word_cells.push(main_cells);

        word_cells
    }

    pub fn index_to_coords(&self, idx: usize) -> (usize, usize) {
        (idx % self.size.x, idx / self.size.y)
    }

    pub fn coords_to_index(x: usize, y: usize, col_len: usize) -> usize {
        y * col_len + x
    }
}

impl View for Board {
    fn draw(&self, printer: &Printer) {
        for (y, row) in self.cells.chunks(self.size.y).enumerate() {
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

        // Print the focused cell
        let Vec2 { x, y } = *self.focus();
        printer.with_color(ColorStyle::highlight(), |printer| {
            if let Some(ch) = self.focused_letter() {
                printer.print((4 * x, y), &format!("[{} ]", ch));
            } else {
                printer.print((x * Cell::size(), y), &format!("{}", self.focused_cell()));
            }
        })
    }

    fn required_size(&mut self, _: Vec2) -> Vec2 {
        self.size.map_x(|x| x * 4)
    }
}

impl Cell {
    pub fn clear_letter(&mut self) -> Option<char> {
        mem::take(&mut self.ch)
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
