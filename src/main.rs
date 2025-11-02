use common::*;

use std::rc::Rc;

use inkview::Event;

fn main() {
    let iv = Box::leak(Box::new(inkview::load())) as &_;

    let (evt_tx, evt_rx) = std::sync::mpsc::channel();

    std::thread::spawn({
        move || {
            if evt_rx.recv().unwrap() != Event::Init {
                panic!("expected init event first");
            }

            // I hope this thing lives as long as the process
            let screen = inkview::screen::Screen::new(iv);

            let display = inkview_slint::Backend::new(screen, evt_rx);

            slint::platform::set_platform(Box::new(display)).unwrap();

            let session = init_session("/mnt/ext1/applications/pbanki/collection");

            let ui = Rc::new(MainWindow::new().unwrap());

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

            ui.run().unwrap();
        }
    });

    inkview::iv_main(iv, {
        move |evt| {
            if evt_tx.send(evt).is_err() {
                unsafe {
                    iv.CloseApp();
                }
            }

            Some(())
        }
    })
}
