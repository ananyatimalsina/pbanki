use std::fs;
use std::rc::Rc;

use anki::{collection::CollectionBuilder, prelude::I18n};
use anki_proto::decks::DeckTreeNode;

use inkview::Event;
use slint::ComponentHandle;
use slint::Model;

mod ui {
    slint::include_modules!();
}

use crate::ui::DeckNode;

fn flatten_tree(node: &DeckTreeNode) -> std::rc::Rc<slint::VecModel<DeckNode>> {
    let result = std::rc::Rc::new(slint::VecModel::<DeckNode>::default());
    flatten_tree_recursive(node, 0, -1, &result);
    result
}

fn flatten_tree_recursive(
    node: &DeckTreeNode,
    level: i32,
    parent_index: i32,
    result: &std::rc::Rc<slint::VecModel<DeckNode>>,
) {
    for child in &node.children {
        let current_index = result.row_count() as i32;

        result.push(DeckNode {
            name: child.name.clone().into(),
            level,
            collapsed: child.collapsed,
            new: child.new_count as i32,
            learn: child.learn_count as i32,
            due: child.review_count as i32,
            has_children: !child.children.is_empty(),
            parent_index,
        });
        flatten_tree_recursive(child, level + 1, current_index, result);
    }
}

fn main() {
    let iv = Box::leak(Box::new(inkview::load())) as &_;

    let (evt_tx, evt_rx) = std::sync::mpsc::channel();

    if let Err(e) = fs::create_dir_all("/mnt/ext1/applications/pbanki/collection/collection.media")
    {
        eprintln!("Failed to create directories: {:?}", e);
        return;
    }

    // Create Collection directly instead of using Backend
    let mut col =
        match CollectionBuilder::new("/mnt/ext1/applications/pbanki/collection/collection.anki2")
            .set_media_paths(
                "/mnt/ext1/applications/pbanki/collection/collection.media/",
                "/mnt/ext1/applications/pbanki/collection/collection.media.db2",
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

    // Now you can use DecksService methods directly on Collection
    let deck_tree = match col.deck_tree(None) {
        Ok(tree) => tree,
        Err(e) => {
            eprintln!("Failed to get deck tree: {:?}", e);
            return;
        }
    };

    std::thread::spawn({
        move || {
            if evt_rx.recv().unwrap() != Event::Init {
                panic!("expected init event first");
            }

            // I hope this thing lives as long as the process
            let screen = inkview::screen::Screen::new(iv);

            let display = inkview_slint::Backend::new(screen, evt_rx);

            slint::platform::set_platform(Box::new(display)).unwrap();

            let window = Rc::new(ui::MainWindow::new().unwrap());

            let deck_model = flatten_tree(&deck_tree);
            window.set_deck_tree(deck_model.into());

            window.run().unwrap();
        }
    });

    inkview::iv_main(iv, {
        move |evt| {
            // println!("got evt: {:?}", evt);

            if evt_tx.send(evt).is_err() {
                unsafe {
                    iv.CloseApp();
                }
            }

            Some(())
        }
    })
}
