use crate::scrabble_event::ScrabbleEvent;
use crate::{board::ScrabbleBoard, direction::Direction};

use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{self, BufRead},
};

use cursive::{
    event::{Event, EventResult},
    theme::Effect,
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
        self.dict.contains(&word.bytes().collect::<Vec<u8>>())
    }

    fn score_of(letter: char) -> u8 {
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

    fn validate_and_score(&mut self) -> Result<HashMap<String, usize>, String> {
        let player = &self.players[self.current_player];

        if self.board.tentative.iter().is_empty() {
            return Err("No letters placed.".to_string());
        }

        let scores = HashMap::new();

        Ok(scores)
    }

    fn score(&mut self, mut letters: Vec<Vec2>, alignment: Alignment) -> (Vec<String>, usize) {
        (vec!["TODO".to_string()], 0)
    }

    fn next_turn(&mut self) {
        let curr_player = &mut self.players[self.current_player];
        self.board.place_tentative();
        // clear those letters from the player pieces and get new ones
        for _ in 0..(N_LETTERS - curr_player.letters.len()) {
            if let Some(letter) = self.letters_bag.pop() {
                curr_player.letters.push(letter);
            }
        }

        self.current_player += 1;
        self.current_player %= self.players.len();
        self.turn += 1;
    }

    fn maybe_toggle_letter(&mut self, ch: char) {
        if self.board.focused_char().is_some()
            || self.board.tentative.contains_key(self.board.focus())
        {
            return;
        }
        if let Some(idx) = self
            .current_player()
            .letters
            .iter()
            .position(|&p_ch| p_ch == ch)
        {
            self.board.tentative.insert(self.board.focus().clone(), ch);
            self.current_player_mut().letters.remove(idx);
        }
    }

    fn remove_focused(&mut self) {
        if let Some(&ch) = self.board.tentative.get(self.board.focus()).clone() {
            self.current_player_mut().letters.push(ch);
            let focus = self.board.focus().clone();
            self.board.tentative.remove(&focus);
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
            printer.print((4 * x + 2, board.y + 2), &String::from(*ch));
            printer.print((4 * x + 4, board.y + 2), "|");
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
            ScrabbleEvent::Letter(ch) => self.maybe_toggle_letter(ch.to_ascii_uppercase()),
            ScrabbleEvent::Delete => self.remove_focused(),
            ScrabbleEvent::Confirm => match self.validate_and_score() {
                Ok(scores) => {
                    self.current_player_mut().add_score(scores.values().sum());
                    self.last_log = format!(
                        "Player {} played {:?}",
                        self.current_player + 1,
                        scores.keys().collect::<Vec<&String>>()
                    );
                    self.next_turn();
                }
                Err(e) => self.last_log = e.to_string(),
            },
            ScrabbleEvent::Undo => todo!(),
            ScrabbleEvent::Redo => todo!(),
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

#[derive(PartialEq, Eq, Clone, Copy)]
enum Alignment {
    Horizontal,
    Vertical,
    Diagonal,
}

impl Alignment {
    fn new(a: &Vec2, b: &Vec2) -> Self {
        if a.x != b.x && a.y != b.y {
            Self::Diagonal
        } else if a.x == b.x {
            Self::Vertical
        } else {
            Self::Horizontal
        }
    }
}
