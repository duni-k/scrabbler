use crate::{
    board::{Board, Cell, Direction, Multiplier},
    event::SEvent,
    gaddag::Gaddag,
    solver::Solver,
};

use cursive::{
    event::{Callback, Event, EventResult},
    view::CannotFocus,
    views::Dialog,
    Vec2,
};

use itertools::Itertools;
use rand::prelude::SliceRandom;

const N_LETTERS: usize = 7;

type PlayerIndex = usize;

pub struct Game {
    board: Board,
    current_player: PlayerIndex,
    dict: Gaddag,
    letters_bag: Vec<char>,
    log: Vec<String>,
    passes: usize,
    players: Vec<Player>,
    turn: usize,
}

#[derive(Clone, Copy)]
pub struct Options {
    pub n_players: usize,
}

impl Game {
    pub fn new(dict: Gaddag, player_names: &[String]) -> Self {
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
            board: Board::new(),
            current_player: 0,
            dict,
            letters_bag: letters,
            log: vec!["Game started! Good luck :)".to_string()],
            passes: 0,
            players,
            turn: 0,
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

    fn validate_placement(&mut self) -> Result<Vec<Vec<Cell>>, String> {
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

    // Returns words and their scores if dictionary contains words, otherwise returns
    // all the words that are not in the dictionary
    fn try_score(
        &mut self,
        word_squares: &Vec<Vec<Cell>>,
    ) -> Result<Vec<(String, usize)>, Vec<String>> {
        let mut words_and_scores = Vec::new();
        let mut not_accepted = Vec::new();
        for squares in word_squares {
            let word = squares.iter().map(|sq| sq.ch.unwrap()).collect::<String>();
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

        if not_accepted.is_empty() {
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
            Ok(words_and_scores)
        } else {
            Err(not_accepted)
        }
    }

    fn next_turn(&mut self) {
        let curr_player = &mut self.players[self.current_player];
        // check BINGO
        let letters_placed = N_LETTERS - curr_player.letters.len();
        if letters_placed == N_LETTERS {
            curr_player.add_score(50);
        }
        // add new letters for player
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
        if self.board.focused_letter().is_some()
            && !self.board.tentative.contains(self.board.focus())
        {
            self.log.push("Cell occupied".to_string());
            return;
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
            let focused = self.board.focused_letter().unwrap().clone();
            self.current_player_mut().letters.push(focused);
            self.board.tentative.remove(&self.board.focus().clone());
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
        self.letters_bag
            .append(&mut self.board.clear_tentative_from_board());
        self.letters_bag.shuffle(&mut rand::thread_rng());
        for _ in 0..amount {
            if let Some(letter) = self.letters_bag.pop() {
                self.current_player_mut().letters.push(letter);
            }
        }
        self.next_turn();
    }

    //  Returns a vector of tuples where the first element is the placement of the player,
    //  the second element element is the player name,
    //  and the third element the player's score.
    fn rank_end_scores(&self) -> Vec<(usize, String, isize)> {
        self.players
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
            .fold(Vec::new(), |mut ranking, (p_name, p_score)| {
                if let Some(&(prev_rank, _, prev_p_score)) = ranking.last() {
                    if prev_p_score == p_score {
                        ranking.push((prev_rank, p_name, p_score));
                    } else {
                        ranking.push((prev_rank + 1, p_name, p_score));
                    }
                } else {
                    ranking.push((1, p_name, p_score))
                }
                ranking
            })
    }

    fn suggest_placement(&mut self) {
        let mut cleared = self.board.clear_tentative_from_board();
        self.current_player_mut().letters.append(&mut cleared);

        let solver = Solver::new(
            &self.board,
            self.current_player().letters.clone(),
            &self.dict,
        );
    }
}

impl cursive::View for Game {
    fn draw(&self, printer: &cursive::Printer) {
        let board = self.board.size;
        let square_size = Cell::size();
        self.board.draw(printer);
        printer.print_hline(board.keep_y().map_y(|y| y), board.x * square_size, "—");
        printer.print(
            (0, board.y + 1),
            &format!("{}'s turn. Letters:", self.current_player().name),
        );

        // Print player letters
        let letter_disp_len = 6;
        let letter_disp_offset = 2;
        printer.print((0, board.y + letter_disp_len), &String::from("|"));
        for (x, ch) in self.current_player().letters.iter().enumerate() {
            printer.print(
                (
                    letter_disp_len * x + letter_disp_offset,
                    board.y + letter_disp_offset,
                ),
                &format!("{ch} {}", Self::score_of(*ch)),
            );
            printer.print(
                (
                    letter_disp_len * x + letter_disp_len,
                    board.y + letter_disp_offset,
                ),
                "|",
            );
        }
        printer.print(
            (
                letter_disp_len * self.current_player().letters.len() + letter_disp_offset,
                board.y + letter_disp_offset,
            ),
            "->",
        );
        for (x, pos) in self.board.tentative.iter().enumerate() {
            let ch = self.board.letter_at(&pos).unwrap();
            printer.with_effect(cursive::theme::Effect::Dim, |printer| {
                printer.print(
                    (
                        x * letter_disp_len
                            + 3
                            + (self.current_player().letters.len() * letter_disp_len
                                + letter_disp_offset),
                        board.y + letter_disp_offset,
                    ),
                    &format!("{ch} {}", Self::score_of(ch)),
                );
                printer.print(
                    (
                        x * letter_disp_len
                            + 7
                            + (self.current_player().letters.len() * letter_disp_len
                                + letter_disp_offset),
                        board.y + letter_disp_offset,
                    ),
                    "|",
                );
            });
        }

        // Print log
        printer.print_hline(board.keep_y().map_y(|y| y + 3), board.x * square_size, "—");
        let mut lines = 0;
        for entry in self.log.iter().rev() {
            printer.print((0, board.y + square_size + lines), "-");
            for line in entry
                .chars()
                .collect::<Vec<char>>()
                .chunks(board.x * square_size - 2)
            {
                printer.print(
                    (2, board.y + square_size + lines),
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
        match SEvent::from(event) {
            SEvent::Move(direction) => {
                self.board.move_focus(&direction);
                self.current_player_mut().previous_move = Some(direction);
            }
            SEvent::Letter(ch) => self.maybe_toggle_letter(ch.to_ascii_uppercase()).to_owned(),
            SEvent::Delete => self.remove_focused(),
            SEvent::Confirm => match self.validate_placement() {
                Ok(word_squares) => {
                    if let Err(e) = self.try_score(&word_squares) {
                        self.log.push(format!("{:#?}", e));
                    } else {
                        self.board.tentative.clear();
                        self.next_turn();
                    }
                }
                Err(e) => self.log.push(e.to_string()),
            },
            SEvent::Pass => {
                self.passes += 1;
                if self.passes >= self.players.len() {
                    let scores_ranked = self.rank_end_scores();
                    return EventResult::Consumed(Some(Callback::from_fn(move |s| {
                        s.pop_layer();
                        s.add_layer(
                            Dialog::new().title("GAME OVER").content(Dialog::info(
                                scores_ranked
                                    .iter()
                                    .map(|(rank, name, score)| {
                                        format!("{rank}: {name} scored {score} points.")
                                    })
                                    .join("\n"),
                            )),
                        );
                    })));
                }
                self.log
                    .push(format!("{} passed their turn.", self.current_player().name));
                let mut cleared = self.board.clear_tentative_from_board();
                self.current_player_mut().letters.append(&mut cleared);
                self.next_turn();
            }
            SEvent::Shuffle => self.current_player_mut().shuffle_letters(),
            SEvent::Suggest => self.suggest_placement(),
            SEvent::Exchange => self.exchange_letters(),
            SEvent::DeleteAll => {
                let cleared = &mut self.board.clear_tentative_from_board();
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
    score: usize,
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

    fn add_score(&mut self, score: usize) {
        self.score += score;
    }
    fn shuffle_letters(&mut self) {
        self.letters.shuffle(&mut rand::thread_rng());
    }
}
