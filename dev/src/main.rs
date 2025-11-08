use common::*;

fn main() {
    let session = init_session("./pbanki/collection");

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

    ui.on_rate(move |rating, deck, chars_per_page| {
        let next = rate_card(&session_for_rate, rating, deck, chars_per_page);
        if let Some(ui) = ui_weak_for_rate.upgrade() {
            ui.set_current_card(next);
        }
    });

    let session_for_deck = session.clone();
    let ui_weak_for_deck = ui.as_weak();

    ui.on_deck_clicked(move |deck, chars_per_page| {
        let next = next_card(&session_for_deck, deck, chars_per_page);
        if let Some(ui) = ui_weak_for_deck.upgrade() {
            ui.set_current_card(next);
        }
    });

    ui.set_deck_tree(update_deck_tree(&session));

    let _ = ui.run();
}

