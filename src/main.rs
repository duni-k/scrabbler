use scrabbler::game::ScrabbleGame;
use std::{
    error::Error,
    fs::{self, File},
    io::{self, BufRead},
    process,
};

use cursive::{
    align::HAlign,
    view::{Nameable, Resizable},
    views::{Button, Dialog, DummyView, EditView, LinearLayout, Panel, SelectView},
    Cursive,
};
use fst::Set;
use serde_derive::Deserialize;
use tokio;

#[derive(Deserialize)]
struct Config {
    lang_file: String,
    players: Vec<PlayerProfile>,
}

#[derive(Deserialize, Clone)]
struct PlayerProfile {
    name: String,
}

#[tokio::main]
async fn main() {
    let conf: Config =
        toml::from_str(&fs::read_to_string("scrabble_config.toml").unwrap()).unwrap();
    let dict = match build_dict(&conf.lang_file).await {
        Err(e) => {
            println!("Could not build dict: {}", e);
            process::exit(1);
        }
        Ok(dict) => dict,
    };

    let mut siv = cursive::default();
    siv.add_layer(
        Dialog::new()
            .title("SCRABBLER")
            .content(
                LinearLayout::vertical()
                    .child(Button::new_raw("New game", move |s| {
                        new_game(s, dict.clone(), conf.players.clone())
                    }))
                    .child(Button::new_raw("How to play", help))
                    .child(Button::new_raw("Exit", Cursive::quit)),
            )
            .h_align(HAlign::Center),
    );
    help(&mut siv);
    siv.add_global_callback('?', help);
    siv.add_global_callback('[', |s| {
        s.pop_layer();
    });

    siv.run();
}

fn help(siv: &mut Cursive) {
    siv.add_layer(Dialog::info(
        "
Welcome to Scrabbler!

Controls:
Use arrow keys or <[HJKL]> to navigate.
Press <Enter> to attempt placement.
Ctrl+e will exchange letters currently placed with random from the bag.
Ctrl+d will delete all letters currently in tentative placement.
Ctrl+p will pass the turn.

? to bring up this screen during game.",
    ));
}

fn new_game(siv: &mut Cursive, dict: Set<Vec<u8>>, player_profiles: Vec<PlayerProfile>) {
    let buttons = LinearLayout::vertical()
        .child(Button::new("New player", add_player))
        .child(Button::new("Delete", delete_player))
        .child(DummyView)
        .child(Button::new("Start game", move |s| {
            if let Some(player_names) =
                &s.call_on_name("select-players", |view: &mut SelectView<String>| {
                    view.iter()
                        .map(|(_, content)| content.clone())
                        .collect::<Vec<String>>()
                })
            {
                if !player_names.is_empty() {
                    start_game(s, ScrabbleGame::new(dict.clone(), player_names))
                }
            }
        }))
        .child(Button::new("Back", |s| {
            s.pop_layer();
        }));
    let select = SelectView::<String>::new()
        .with_all_str(player_profiles.iter().map(|p| p.name.clone()))
        .with_name("select-players")
        .fixed_size((10, 5));

    siv.add_layer(
        Dialog::around(
            LinearLayout::horizontal()
                .child(buttons)
                .child(DummyView)
                .child(select),
        )
        .title("Select players"),
    );
}

fn add_player(s: &mut Cursive) {
    fn ok(s: &mut Cursive, name: &str) {
        s.call_on_name("select-players", |view: &mut SelectView<String>| {
            view.add_item_str(name)
        });
        s.pop_layer();
    }

    s.add_layer(
        Dialog::around(
            EditView::new()
                .on_submit(ok)
                .with_name("name")
                .fixed_width(10),
        )
        .title("Enter a new name")
        .button("Ok", |s| {
            let name = s
                .call_on_name("name", |view: &mut EditView| view.get_content())
                .unwrap();
            ok(s, &name);
        })
        .button("Cancel", |s| {
            s.pop_layer();
        }),
    );
}

fn delete_player(s: &mut Cursive) {
    let mut select = s.find_name::<SelectView<String>>("select-players").unwrap();
    if let Some(focus) = select.selected_id() {
        select.remove_item(focus);
    }
}

fn start_game(siv: &mut Cursive, game: ScrabbleGame) {
    siv.add_layer(
        Dialog::new()
            .title("SCRABBLER")
            .content(LinearLayout::horizontal().child(Panel::new(game)))
            .button("New game", |s| {
                s.pop_layer();
            })
            .button("Quit", |s| {
                s.pop_layer();
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
