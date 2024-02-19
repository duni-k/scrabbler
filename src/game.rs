use crate::scrabble_event::ScrabbleEvent;
use crate::{
    board::{Multiplier, ScrabbleBoard, Square},
    direction::Direction,
};

use cursive::{
    event::{Callback, Event, EventResult},
    view::CannotFocus,
    views::Dialog,
    Vec2,
};
use fst::Set;
use itertools::Itertools;
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
    log: Vec<String>,
    turn: usize,
    passes: usize,
}

#[derive(Clone, Copy)]
pub struct Options {
    pub n_players: usize,
}

impl ScrabbleGame {
    pub fn new(dict: Set<Vec<u8>>, player_names: &Vec<String>) -> Self {
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

        let mut players = Vec::new();
        for name in player_names {
            let player_letters = letters.drain(0..N_LETTERS).collect();
            players.push(Player::new(player_letters, name.clone()));
        }

        Self {
            board: ScrabbleBoard::new(),
            players,
            letters_bag: letters,
            dict,
            log: vec!["Game started! Good luck :)".to_string()],
            ..Default::default()
        }
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

        if self.turn == 0 {
            let center = self.board.size.map(|v| (v - 1) / 2);
            if !self.board.tentative.contains(&center) {
                return Err("First placement must contain center square.".to_string());
            }
        } else if !self.board.is_connected() {
            return Err("Letters not connected to existing.".to_string());
        }

        Ok(self.board.collect_tentative()?)
    }

    fn score(&mut self, word_squares: &Vec<Vec<Square>>) -> Result<Vec<(String, usize)>, String> {
        let mut words_and_scores = Vec::new();
        let mut not_accepted = Vec::new();
        for squares in word_squares {
            let word = squares.iter().map(|sq| sq.ch.unwrap()).collect();
            if !self.dict.contains(&word) {
                not_accepted.push(word);
                continue;
            }
            let mut word_score = 0;
            let mut word_mults = Vec::new();
            for square in squares {
                let letter_score = Self::score_of(square.ch.unwrap());
                word_score += match square.mult {
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
                    .fold(word_score, |acc, mult| acc * mult.as_factor()),
            ));
        }

        if !not_accepted.is_empty() {
            Err(format!("Word(s) not accepted: {:?}.", not_accepted))
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

        self.current_player += 1;
        if self.current_player >= self.players.len() {
            self.current_player = 0;
            self.passes = 0;
        }
        self.turn += 1;
    }

    fn maybe_toggle_letter(&mut self, ch: char) {
        if self.board.focused_char().is_some() && !self.board.tentative.contains(self.board.focus())
        {
            self.log.push("Square occupied".to_string());
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
            self.log
                .push("No such letter belonging to player.".to_string())
        }
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

    fn exchange_letters(&mut self) {
        if self.board.tentative.len() > self.letters_bag.len() {
            self.log
                .push("Can't exchange more letters than are left in bag.".to_string());
            return;
        }
        let amount = self.board.tentative.len();
        self.letters_bag.append(&mut self.board.clear_tentative());
        self.letters_bag.shuffle(&mut rand::thread_rng());
        for _ in 0..amount {
            if let Some(letter) = self.letters_bag.pop() {
                self.current_player_mut().letters.push(letter);
            }
        }
        self.next_turn();
    }

    fn rank_end_scores(&self) -> Vec<(usize, (String, isize))> {
        let mut scores_ranked: Vec<(usize, (String, isize))> = self
            .players
            .iter()
            .map(|p| {
                (
                    p.name.clone(),
                    p.score as isize
                        - p.letters
                            .iter()
                            .map(|&letter| Self::score_of(letter) as isize)
                            .sum::<isize>(),
                )
            })
            .sorted_unstable_by_key(|(_, score)| score.clone())
            .enumerate()
            .collect();

        for i in 0..scores_ranked.len() {
            if let Some(&(_, (_, score2))) = scores_ranked.get(i + 1) {
                if scores_ranked[i].1 .1 == score2 {
                    scores_ranked[i + 1].0 = i;
                } else {
                    scores_ranked[i + 1].0 = scores_ranked[i].0 + 1;
                }
            }
        }

        scores_ranked
    }
}

impl cursive::View for ScrabbleGame {
    fn draw(&self, printer: &cursive::Printer) {
        let board = self.board.size;
        self.board.draw(printer);
        printer.print_hline(board.keep_y().map_y(|y| y), board.x * 4, "—");
        printer.print(
            (0, board.y + 1),
            &format!("{}'s turn. Letters:", self.current_player().name),
        );

        // Print player letters
        printer.print((0, board.y + 2), &String::from("|"));
        for (x, ch) in self.current_player().letters.iter().enumerate() {
            printer.print(
                (6 * x + 2, board.y + 2),
                &format!("{} {}", ch, Self::score_of(*ch)),
            );
            printer.print((6 * x + 6, board.y + 2), "|");
        }
        printer.print(
            (6 * self.current_player().letters.len() + 2, board.y + 2),
            "->",
        );
        for (x, pos) in self.board.tentative.iter().enumerate() {
            let ch = self.board.letter_at(&pos).unwrap();
            printer.with_effect(cursive::theme::Effect::Dim, |printer| {
                printer.print(
                    (
                        6 * x + 3 + (self.current_player().letters.len() * 6 + 2),
                        board.y + 2,
                    ),
                    &format!("{} {}", ch, Self::score_of(ch)),
                );
                printer.print(
                    (
                        6 * x + 7 + (self.current_player().letters.len() * 6 + 2),
                        board.y + 2,
                    ),
                    "|",
                );
            });
        }

        // Print log
        printer.print_hline(board.keep_y().map_y(|y| y + 3), board.x * 4, "—");
        let mut lines = 0;
        for entry in self.log.iter().rev() {
            printer.print((0, board.y + 4 + lines), "-");
            for line in entry.chars().collect::<Vec<char>>().chunks(board.x * 4 - 2) {
                printer.print(
                    (2, board.y + 4 + lines),
                    &line.into_iter().collect::<String>(),
                );
                lines += 1;
            }
        }

        // Print player scores
        let player_window_x = board.x * 4 + 2;
        for (i, player) in self.players.iter().enumerate() {
            printer.with_effect(
                if i == self.current_player {
                    cursive::theme::Effect::Underline
                } else {
                    cursive::theme::Effect::Dim
                },
                |printer| {
                    printer.print((player_window_x, i * 3), &format!("{}", player.name));
                },
            );
            printer.print(
                (player_window_x, i * 3 + 1),
                &format!("{} pts", player.score),
            );
            printer.print_hline((player_window_x, i * 3 + 2), 10, "-");
        }
    }

    fn required_size(&mut self, _: Vec2) -> Vec2 {
        self.board.size.map_x(|x| x * 4 + 12).map_y(|y| y + 10)
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        match ScrabbleEvent::from(event) {
            ScrabbleEvent::Move(direction) => {
                self.board.move_focus(&direction);
                self.current_player_mut().previous_move = Some(direction);
            }
            ScrabbleEvent::Letter(ch) => {
                self.maybe_toggle_letter(ch.to_ascii_uppercase()).to_owned()
            }
            ScrabbleEvent::Delete => self.remove_focused(),
            ScrabbleEvent::Confirm => match self.validate_placement() {
                Ok(word_squares) => match self.score(&word_squares) {
                    Ok(words_and_scores) => {
                        let score_tot = words_and_scores.iter().map(|(_, score)| score).sum();
                        self.current_player_mut().add_score(score_tot);
                        self.log.push(if words_and_scores.len() == 1 {
                            format!(
                                "{} played {} for {} points.",
                                self.current_player().name,
                                words_and_scores.iter().next().unwrap().0,
                                score_tot
                            )
                        } else {
                            format!(
                                "{} played {:?}, {} points total.",
                                self.current_player().name,
                                words_and_scores,
                                score_tot,
                            )
                        });
                        self.next_turn();
                    }
                    Err(e) => self.log.push(e),
                },
                Err(e) => self.log.push(e.to_string()),
            },
            ScrabbleEvent::Pass => {
                self.passes += 1;
                if self.passes >= self.players.len() {
                    let scores_ranked = self.rank_end_scores();
                    return EventResult::Consumed(Some(Callback::from_fn(move |s| {
                        s.pop_layer();
                        s.add_layer(
                            Dialog::new().title("GAME OVER").content(Dialog::info(
                                scores_ranked
                                    .iter()
                                    .map(|(i, (name, score))| {
                                        format!("{}: {} scored {} points.", i + 1, name, score)
                                    })
                                    .collect::<Vec<String>>()
                                    .join("\n"),
                            )),
                        );
                    })));
                }
                self.log
                    .push(format!("{} passed their turn.", self.current_player().name));
                self.next_turn();
            }
            ScrabbleEvent::Exchange => self.exchange_letters(),
            ScrabbleEvent::DeleteAll => {
                let cleared = &mut self.board.clear_tentative();
                self.current_player_mut().letters.append(cleared);
            }
            _ => return EventResult::Ignored,
        };

        EventResult::Consumed(None)
    }

    fn take_focus(&mut self, _: cursive::direction::Direction) -> Result<EventResult, CannotFocus> {
        Ok(EventResult::Consumed(None))
    }
}

struct Player {
    name: String,
    letters: Vec<char>,
    score: ScrabbleScore,
    previous_move: Option<Direction>,
}

impl Player {
    fn new(chars: Vec<char>, name: String) -> Self {
        Self {
            letters: chars,
            score: 0,
            previous_move: None,
            name,
        }
    }

    fn add_score(&mut self, score: ScrabbleScore) {
        self.score += score;
    }
}
