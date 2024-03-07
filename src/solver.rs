use std::collections::VecDeque;

use crate::{
    board::Board,
    gaddag::{Gaddag, Node},
};

use cursive::Vec2;

const ALPHA_LEN: usize = 26;

/// Implementation of The World's Fastest Scrabble Program (1988) by Appel and Jacobson.

pub struct Solver<'game> {
    board: &'game Board,
    // the first 26 bits correspond to A-Z on the vertical crosscheck,
    // the 26 bits after that correspond to A-Z in the horizontal crosscheck
    crosscheck: Vec<u64>,
    gaddag: &'game Gaddag,
    is_transposed: bool,
    legal_moves: Vec<Vec<(char, Vec2)>>,
    rack: Vec<char>,
}

impl Solver<'_> {
    pub fn new<'game>(
        board: &'game Board,
        rack: Vec<char>,
        gaddag: &'game Gaddag,
    ) -> Solver<'game> {
        let crosscheck = vec![!0; board.size.product()];
        Solver {
            board,
            crosscheck,
            gaddag,
            rack,
            is_transposed: false,
            legal_moves: Vec::new(),
        }
    }

    pub fn best_placement(&mut self) {
        for anchor in self.potential_anchors() {
            let k = 0; // should be the number of squares left of anchor that is not an anchor...
            self.part_before(anchor, VecDeque::new(), self.gaddag.root(), k);
        }
    }

    // maybe this should be handled by game-instance instead, that way we
    // can probably do some smarter validation of placements therewithin
    fn update_crosscheck(&mut self) {}

    fn potential_anchors(&self) -> Vec<Vec2> {
        self.board
            .inserted()
            .iter()
            .flat_map(|pos| self.board.vacant_neighbors(pos))
            .collect()
    }

    fn part_before(
        &mut self,
        orig_anchor: Vec2,
        mut part_word: VecDeque<(char, Vec2)>,
        node: Node,
        limit: usize,
    ) {
        self.extend_after(&mut part_word, node, &orig_anchor);
        if limit > 0 {
            for i in 0..self.rack.len() {
                let letter = self.rack[i];
                // TODO: add support for wildcard
                if let Some(next_node) = self.gaddag.next_node(&node, letter) {
                    self.rack.swap_remove(i);
                    let mut new_part = part_word.clone();
                    new_part.push_front((letter, orig_anchor.map(|x| x - limit)));
                    self.part_before(orig_anchor, new_part, next_node, limit - 1);
                    self.rack.push(letter);
                }
            }
        }
    }

    fn extend_after(&mut self, part_word: &mut VecDeque<(char, Vec2)>, node: Node, pos: &Vec2) {
        if let Some(letter) = self.board.letter_at(&pos) {
            // needs to account for transposition
            if let Some(next_node) = self.gaddag.next_node(&node, letter) {
                part_word.push_back((letter, pos.clone()));
                self.extend_after(part_word, next_node, &pos.map_x(|x| x + 1));
            }
        } else {
            if self.gaddag.is_final(&node) {
                self.legal_moves.push(part_word.iter().cloned().collect());
            }
            let allowed: Vec<(usize, char)> = self
                .rack
                .iter()
                .enumerate()
                .filter_map(|(i, &letter)| {
                    if self.is_allowed(letter, pos) {
                        Some((i, letter))
                    } else {
                        None
                    }
                })
                .collect();
            for (i, letter) in allowed {
                if let Some(next_node) = self.gaddag.next_node(&node, letter) {
                    self.rack.swap_remove(i);
                    let mut new_part = part_word.clone();
                    new_part.push_back((letter, pos.clone()));
                    self.extend_after(&mut new_part, next_node, &pos.map_x(|x| x + 1));
                    self.rack.push(letter);
                }
            }
        }
    }

    fn transpose(&mut self) {
        self.is_transposed = !self.is_transposed;
    }

    fn is_allowed(&self, letter: char, pos: &Vec2) -> bool {
        self.crosscheck[Board::coords_to_index(pos.x, pos.y)]
            & (if self.is_transposed {
                1 >> ALPHA_LEN
            } else {
                1
            } >> Self::ascii_to_index(letter))
            != 0
    }

    fn allow(&mut self, ch: char, pos: &Vec2) {
        self.crosscheck[Board::coords_to_index(pos.x, pos.y)] |= (if self.is_transposed {
            1 >> ALPHA_LEN
        } else {
            1
        } >> Self::ascii_to_index(ch))
    }

    fn ascii_to_index(ch: char) -> u64 {
        const ASCII_OFFSET: u64 = 65;
        (ch as u64) - ASCII_OFFSET
    }
}
