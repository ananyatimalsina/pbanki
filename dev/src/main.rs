mod api;
mod utils;

use crate::api::{LearnSession, next_card, rate_card, update_deck_tree};

use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

use anki::{collection::CollectionBuilder, prelude::I18n};

slint::include_modules!();

fn main() {
    if let Err(e) = fs::create_dir_all("./pbanki/collection/collection.media") {
        eprintln!("Failed to create directories: {:?}", e);
        return;
    }

    // Create Collection directly instead of using Backend
    let col = match CollectionBuilder::new("./pbanki/collection/collection.anki2")
        .set_media_paths(
            "./pbanki/collection/collection.media/",
            "./pbanki/collection/collection.media.db2",
        )
        .set_tr(I18n::new(&["en"]))
        .build()
    {
        Ok(col) => col,
        Err(e) => {
            eprintln!("Failed to open collection: {:?}", e);
            return;
        }
    };

    let col = Rc::new(RefCell::new(col));

    let session = Rc::new(LearnSession {
        collection: col.clone(),
        current_card: RefCell::new(CardNode::default()),
        states: RefCell::new(None),
        start_time: RefCell::new(None),
    });

    let ui = MainWindow::new().unwrap();

    let session_for_deck_tree = session.clone();
    let ui_weak_for_deck_tree = ui.as_weak();
    ui.on_update_deck_tree(move || {
        let deck_tree = update_deck_tree(&session_for_deck_tree);
        if let Some(ui) = ui_weak_for_deck_tree.upgrade() {
            ui.set_deck_tree(deck_tree);
        }
    });

    let session_for_rate = session.clone();
    let ui_weak_for_rate = ui.as_weak();

    ui.on_rate(move |rating, deck| {
        let next = rate_card(&session_for_rate, rating, deck);
        if let Some(ui) = ui_weak_for_rate.upgrade() {
            ui.set_current_card(next);
        }
    });

    let session_for_deck = session.clone();
    let ui_weak_for_deck = ui.as_weak();

    ui.on_deck_clicked(move |deck| {
        let next = next_card(&session_for_deck, deck);
        if let Some(ui) = ui_weak_for_deck.upgrade() {
            ui.set_current_card(next);
        }
    });

    ui.set_deck_tree(update_deck_tree(&session));

    let _ = ui.run();
}
