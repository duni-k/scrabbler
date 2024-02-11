use crate::scrabble_event::ScrabbleEvent;
use crate::{
    board::{Multiplier, ScrabbleBoard, Square},
    direction::Direction,
};

use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{self, BufRead},
};

use cursive::{
    event::{Event, EventResult},
    view::CannotFocus,
    Vec2,
};
use fst::Set;
use rand::prelude::SliceRandom;

const N_LETTERS: usize = 7;

type PlayerIndex = usize;
type ScrabbleScore = usize;

#[derive(Default)]
pub struct ScrabbleGame {
    board: ScrabbleBoard,
    dict: Set<Vec<u8>>,
    players: Vec<Player>,
    current_player: PlayerIndex,
    letters_bag: Vec<char>,
    last_log: String,
    turn: usize,
    passes: usize,
}

impl ScrabbleGame {
    pub fn new(n_players: usize) -> Result<Self, &'static str> {
        let mut letters = vec![
            vec!['A'; 9],
            vec!['B'; 2],
            vec!['C'; 2],
            vec!['D'; 4],
            vec!['E'; 12],
            vec!['F'; 2],
            vec!['G'; 3],
            vec!['H'; 2],
            vec!['I'; 9],
            vec!['J'; 1],
            vec!['K'; 1],
            vec!['L'; 4],
            vec!['M'; 2],
            vec!['N'; 6],
            vec!['O'; 8],
            vec!['P'; 2],
            vec!['Q'; 1],
            vec!['R'; 6],
            vec!['S'; 4],
            vec!['T'; 6],
            vec!['U'; 4],
            vec!['V'; 2],
            vec!['W'; 2],
            vec!['X'; 1],
            vec!['Y'; 2],
            vec!['Z'; 1],
            vec![' '; 2],
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<char>>();
        letters.shuffle(&mut rand::thread_rng());

        let mut players = Vec::with_capacity(n_players);
        for _ in 0..n_players {
            let mut player_letters = Vec::new();
            for _ in 0..N_LETTERS {
                if let Some(ch) = letters.pop() {
                    player_letters.push(ch);
                } else {
                    return Err("Too many players :/");
                }
            }
            players.push(Player::new(player_letters));
        }

        Ok(Self {
            board: ScrabbleBoard::new(),
            players,
            letters_bag: letters,
            ..Default::default()
        })
    }

    pub async fn build_dict(&mut self, file_name: &String) -> Result<(), Box<dyn Error>> {
        let reader = io::BufReader::new(File::open(file_name)?);
        Ok(self.dict = Set::from_iter(
            reader
                .lines()
                .map(|l| l.unwrap_or("".into()).trim().to_owned()),
        )?)
    }

    pub fn accepts(&self, word: &String) -> bool {
        self.dict.contains(word)
    }

    fn score_of(letter: char) -> usize {
        match letter {
            'A' | 'E' | 'I' | 'L' | 'N' | 'O' | 'R' | 'S' | 'T' | 'U' => 1,
            'D' | 'G' => 2,
            'B' | 'C' | 'M' | 'P' => 3,
            'F' | 'H' | 'V' | 'W' | 'Y' => 4,
            'K' => 5,
            'J' | 'X' => 8,
            'Q' | 'Z' => 10,
            ' ' => 0,
            _ => unreachable!(),
        }
    }

    fn validate_placement(&mut self) -> Result<Vec<Vec<Square>>, String> {
        if self.board.tentative.is_empty() {
            return Err("No letters placed.".to_string());
        }

        if self.turn > 0 && !self.board.is_connected() {
            return Err("Letters not connected to existing.".to_string());
        }

        Ok(self.board.collect_tentative()?)
    }

    fn score(&mut self, word_squares: &Vec<Vec<Square>>) -> Result<Vec<(String, usize)>, String> {
        let mut words_and_scores = Vec::new();
        let mut not_accepted = Vec::new();
        for squares in word_squares {
            let word = squares.iter().map(|sq| sq.ch.unwrap()).collect();
            if !self.accepts(&word) {
                not_accepted.push(word);
                continue;
            }
            let mut score = 0;
            let mut word_mults = Vec::new();
            for square in squares {
                let letter_score = Self::score_of(square.ch.unwrap());
                score += match square.mult {
                    None => letter_score,
                    Some(word_mult @ (Multiplier::Dw | Multiplier::Tw)) => {
                        word_mults.push(word_mult);
                        letter_score
                    }
                    Some(letter_mult @ (Multiplier::Dl | Multiplier::Tl)) => {
                        letter_score * letter_mult.as_factor()
                    }
                };
            }
            words_and_scores.push((
                word,
                word_mults
                    .iter()
                    .fold(score, |acc, mult| acc * mult.as_factor()),
            ));
        }

        if !not_accepted.is_empty() {
            Err(format!("Words not accepted: {:?}.", not_accepted))
        } else {
            Ok(words_and_scores)
        }
    }

    fn next_turn(&mut self) {
        let curr_player = &mut self.players[self.current_player];
        self.board.tentative.clear();
        // check BINGO
        let letters_placed = N_LETTERS - curr_player.letters.len();
        if letters_placed == N_LETTERS {
            curr_player.add_score(50);
        }
        // clear those letters from the player pieces and get new ones
        for _ in 0..letters_placed {
            if let Some(letter) = self.letters_bag.pop() {
                curr_player.letters.push(letter);
            }
        }
        // everyone passed
        if self.passes >= self.players.len() {
            // subtract letter scores of letters still held from player scores
            // announce winner
            // restart possible?
            todo!();
        }

        self.current_player += 1;
        if self.current_player >= self.players.len() {
            self.current_player = 0;
            self.passes = 0;
        }
        self.turn += 1;
    }

    fn maybe_toggle_letter(&mut self, ch: char) -> Result<(), &str> {
        if self.board.focused_char().is_some() && !self.board.tentative.contains(self.board.focus())
        {
            return Err("Other player already placed a letter there.");
        }

        if let Some(idx) = self
            .current_player()
            .letters
            .iter()
            .position(|&p_ch| p_ch == ch)
        {
            if let Some(existing_ch) = self.board.place_focused(ch) {
                self.current_player_mut().letters.push(existing_ch);
            }
            self.board.tentative.insert(self.board.focus().clone());
            self.current_player_mut().letters.remove(idx);
        } else {
            return Err("No such letter belonging to player.");
        }

        Ok(())
    }

    fn remove_focused(&mut self) {
        if self.board.tentative.contains(self.board.focus()) {
            let focused_char = self.board.focused_char().unwrap().clone();
            self.current_player_mut().letters.push(focused_char);
            let focus = &self.board.focus().clone();
            self.board.tentative.remove(focus);
            self.board.clear_focused();
        }
    }

    fn current_player(&self) -> &Player {
        self.players.get(self.current_player).unwrap()
    }

    fn current_player_mut(&mut self) -> &mut Player {
        self.players.get_mut(self.current_player).unwrap()
    }
}

impl cursive::View for ScrabbleGame {
    fn draw(&self, printer: &cursive::Printer) {
        let board = self.board.size;
        self.board.draw(printer);
        printer.print_hline(board.keep_y().map_y(|y| y), board.x * 4, "—");
        printer.print(
            (0, board.y + 1),
            &format!("Player {}'s turn. Letters:", self.current_player + 1),
        );

        printer.print((0, board.y + 2), &String::from("|"));
        for (x, ch) in self.players[self.current_player].letters.iter().enumerate() {
            printer.print(
                (6 * x + 2, board.y + 2),
                &format!("{} {}", ch, Self::score_of(*ch)),
            );
            printer.print((6 * x + 6, board.y + 2), "|");
        }
        printer.print_hline(board.keep_y().map_y(|y| y + 3), board.x * 4, "—");
        printer.print((0, board.y + 4), &self.last_log);
    }

    fn required_size(&mut self, _: Vec2) -> Vec2 {
        self.board.size.map_x(|x| x * 4).map_y(|y| y + 5)
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        match ScrabbleEvent::from(event) {
            ScrabbleEvent::Move(direction) => {
                self.board.move_focus(&direction);
                self.current_player_mut().previous_move = Some(direction);
            }
            ScrabbleEvent::Letter(ch) => {
                if let Err(e) = self.maybe_toggle_letter(ch.to_ascii_uppercase()) {
                    self.last_log = e.to_string();
                }
            }
            ScrabbleEvent::Delete => self.remove_focused(),
            ScrabbleEvent::Confirm => match self.validate_placement() {
                Ok(word_squares) => match self.score(&word_squares) {
                    Ok(words_and_scores) => {
                        let score_tot = words_and_scores.iter().map(|(_, score)| score).sum();
                        self.current_player_mut().add_score(score_tot);
                        self.last_log = format!(
                            "Player {} played {:?}, {} points total.",
                            self.current_player + 1,
                            words_and_scores,
                            score_tot,
                        );
                        self.next_turn();
                    }
                    Err(e) => self.last_log = e,
                },
                Err(e) => self.last_log = e.to_string(),
            },
            ScrabbleEvent::Undo => todo!(),
            ScrabbleEvent::Redo => todo!(),
            ScrabbleEvent::Pass => {
                self.passes += 1;
                self.next_turn();
            }
            ScrabbleEvent::Ignored => return EventResult::Ignored,
        };

        EventResult::Consumed(None)
    }

    fn take_focus(&mut self, _: cursive::direction::Direction) -> Result<EventResult, CannotFocus> {
        Ok(EventResult::Consumed(None))
    }
}

struct Player {
    letters: Vec<char>,
    score: ScrabbleScore,
    previous_move: Option<Direction>,
}

impl Player {
    fn new(chars: Vec<char>) -> Self {
        Self {
            letters: chars,
            score: 0,
            previous_move: None,
        }
    }

    fn add_score(&mut self, score: ScrabbleScore) {
        self.score += score;
    }
}
