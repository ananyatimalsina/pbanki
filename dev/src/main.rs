use std::fs;

use anki::timestamp::TimestampSecs;
use anki::{collection::CollectionBuilder, prelude::I18n};
use anki_proto::decks::DeckTreeNode;

use slint::Model;
slint::include_modules!();

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
    if let Err(e) = fs::create_dir_all("./pbanki/collection/collection.media") {
        eprintln!("Failed to create directories: {:?}", e);
        return;
    }

    // Create Collection directly instead of using Backend
    let mut col = match CollectionBuilder::new("./pbanki/collection/collection.anki2")
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

    // Now you can use DecksService methods directly on Collection
    let deck_tree = match col.deck_tree(Some(TimestampSecs::now())) {
        Ok(tree) => tree,
        Err(e) => {
            eprintln!("Failed to get deck tree: {:?}", e);
            return;
        }
    };

    println!("Deck tree: {:?}", deck_tree);

    let ui = MainWindow::new().unwrap();

    ui.set_due_total(deck_tree.review_count as i32);

    let deck_model = flatten_tree(&deck_tree);
    ui.set_deck_tree(deck_model.into());

    let _ = ui.run();
}
