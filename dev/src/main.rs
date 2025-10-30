use std::fs;

use anki::{collection::CollectionBuilder, prelude::I18n};

slint::include_modules!();

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
    let deck_tree = match col.deck_tree(None) {
        Ok(tree) => tree,
        Err(e) => {
            eprintln!("Failed to get deck tree: {:?}", e);
            return;
        }
    };

    println!("{:?}", deck_tree);

    let ui = MainWindow::new().unwrap();
    let _ = ui.run();
}
