use scrabbler::game::ScrabbleGame;
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead},
    process,
};

use cursive::{
    align::HAlign,
    views::{Button, Dialog, LinearLayout, Panel, SelectView},
    Cursive,
};
use fst::Set;
use tokio;

const DEFAULT_DICT: &'static str = "dict_en.txt";

#[tokio::main]
async fn main() {
    let dict = match build_dict(&DEFAULT_DICT.into()).await {
        Err(e) => {
            println!("Could not build dict: {}", e);
            process::exit(1);
        }
        Ok(dict) => dict,
    };

    let mut siv = cursive::default();
    siv.add_layer(
        Dialog::new().title("SCRABBLER").content(
            LinearLayout::vertical()
                .child(Button::new_raw("  New game   ", move |s| {
                    new_game(s, dict.clone())
                }))
                .child(Button::new_raw("    Exit     ", |s| s.quit())),
        ),
    );
    siv.add_global_callback('?', help);

    siv.run();
}

fn help(siv: &mut Cursive) {
    siv.add_layer(Dialog::info(
        "Welcome to Scrabbler!

    Controls:
    Use arrow keys or <[HJKL]> to navigate.
    Press <Enter> to attempt placement.
    Ctrl+e will exchange letters currently placed with random from the bag.
    Ctrl+d will delete all letters currently in tentative placement.
    Ctrl+p will pass the turn.",
    ));
}

fn new_game(siv: &mut Cursive, dict: Set<Vec<u8>>) {
    siv.add_layer(
        Dialog::new()
            .title("Select number of players")
            .content(
                SelectView::new()
                    .item("2", 2)
                    .item("3", 3)
                    .item("4", 4)
                    .on_submit(move |s, n_players| {
                        s.pop_layer();
                        start_game(s, ScrabbleGame::new(*n_players, dict.clone()));
                    })
                    .h_align(HAlign::Center),
            )
            .dismiss_button("Back"),
    );
}

fn start_game(siv: &mut Cursive, game: ScrabbleGame) {
    siv.add_layer(
        Dialog::new()
            .title("SCRABBLER")
            .content(LinearLayout::horizontal().child(Panel::new(game)))
            .button("Quit", |s| {
                s.pop_layer();
            }),
    );
}

pub async fn build_dict(file_name: &String) -> Result<Set<Vec<u8>>, Box<dyn Error>> {
    let reader = io::BufReader::new(File::open(file_name)?);
    Ok(Set::from_iter(
        reader
            .lines()
            .map(|l| l.unwrap_or("".into()).trim().to_owned()),
    )?)
}
