use scrabbler::game::ScrabbleGame;
use std::{error::Error, process};

use cursive::views::{Dialog, LinearLayout, Panel};
use tokio;

const DEFAULT_DICT: &'static str = "dict_en.txt";
const N_PLAYERS: usize = 2;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut game = ScrabbleGame::new(N_PLAYERS)?;
    if let Err(e) = game.build_dict(&DEFAULT_DICT.into()).await {
        println!("Could not build dict: {}", e);
        process::exit(1);
    }

    let mut siv = cursive::default();

    siv.add_layer(
        Dialog::new()
            .title("SCRABBLER")
            .content(LinearLayout::horizontal().child(Panel::new(game)))
            .button("Quit", |s| s.quit()), //.button("Restart", restart),
    );
    siv.add_layer(Dialog::info(
        "Welcome to Scrabbler!

Controls:
Use arrow keys or <[hjkl]> to navigate.
Use the number keys to insert the corresponding char on the board.
Press <Enter> to attempt move.",
    ));

    siv.run();
    Ok(())
}

// TODO:
// fn help(s: &mut Cursive) {}

// TODO:
// fn restart(s: &mut Cursive) {}
