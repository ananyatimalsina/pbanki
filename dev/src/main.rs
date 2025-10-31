use std::cell::RefCell;
use std::fs;
use std::rc::Rc;
use std::time::Instant;

use anki::timestamp::TimestampSecs;
use anki::{collection::CollectionBuilder, prelude::I18n};
use anki_proto::decks::DeckTreeNode;

use slint::Model;
slint::include_modules!();

struct LearnSession {
    collection: Rc<RefCell<anki::collection::Collection>>,
    current_card: RefCell<CardNode>,
    start_time: RefCell<Option<Instant>>,
}

fn flatten_tree(node: &DeckTreeNode) -> Rc<slint::VecModel<DeckNode>> {
    let result = Rc::new(slint::VecModel::<DeckNode>::default());
    flatten_tree_recursive(node, -1, &result);
    result
}

fn flatten_tree_recursive(
    node: &DeckTreeNode,
    parent_index: i32,
    result: &Rc<slint::VecModel<DeckNode>>,
) {
    for child in &node.children {
        let current_index = result.row_count() as i32;

        result.push(DeckNode {
            id: child.deck_id,
            name: child.name.clone().into(),
            level: child.level as i32,
            collapsed: child.collapsed,
            new: child.new_count as i32,
            learn: child.learn_count as i32,
            due: child.review_count as i32,
            has_children: !child.children.is_empty(),
            parent_index,
        });
        flatten_tree_recursive(child, current_index, result);
    }
}

fn remove_double_brackets(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut skipping = false;

    while let Some(c) = chars.next() {
        if !skipping {
            if c == '[' {
                if let Some(&'[') = chars.peek() {
                    // Found opening [[, start skipping
                    chars.next(); // consume second '['
                    skipping = true;
                    continue;
                } else {
                    output.push(c);
                }
            } else {
                output.push(c);
            }
        } else {
            // Currently skipping until finding ]]
            if c == ']' {
                if let Some(&']') = chars.peek() {
                    chars.next(); // consume second ']'
                    skipping = false;
                }
            }
        }
    }
    output
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

    let col = Rc::new(RefCell::new(col));

    let session = Rc::new(LearnSession {
        collection: col.clone(),
        current_card: RefCell::new(CardNode::default()),
        start_time: RefCell::new(None),
    });

    let ui = MainWindow::new().unwrap();
    let ui_weak = ui.as_weak();

    let session_clone = session.clone();

    ui.on_deck_clicked(move |deck_id| {
        let ui = ui_weak.unwrap();

        let mut col_borrow = session_clone.collection.borrow_mut();
        let _ = col_borrow.set_current_deck(anki::decks::DeckId(deck_id));
        let queued_card = col_borrow.get_next_card().unwrap();
        drop(col_borrow);

        if let Some(card) = queued_card {
            *session_clone.start_time.borrow_mut() = Some(Instant::now());

            let rendered = session_clone
                .collection
                .borrow_mut()
                .render_existing_card(card.card.id(), false, false)
                .unwrap();

            let mut answer = anki::text::strip_html(rendered.answer().as_ref()).into_owned();

            // Handle special case for type in cards
            if answer.contains("[[type:") {
                let note_opt = session_clone
                    .collection
                    .borrow_mut()
                    .storage
                    .get_note(card.card.note_id())
                    .unwrap();

                if let Some(note) = note_opt {
                    let fields = note.fields();
                    let notetype_id = note.notetype_id;
                    let notetype = session_clone
                        .collection
                        .borrow_mut()
                        .get_notetype(notetype_id)
                        .unwrap();

                    if let Some(notetype) = notetype {
                        for (index, field) in notetype.fields.iter().enumerate() {
                            if field.name == "Back" {
                                answer = (&fields[index]).to_owned();
                            }
                        }
                    }
                }
            }

            let card_node = CardNode {
                question: remove_double_brackets(
                    &anki::text::strip_html(rendered.question().as_ref()).into_owned(),
                )
                .into(),
                answer: answer.into(),
            };

            *session_clone.current_card.borrow_mut() = card_node.clone();
            ui.set_current_card(card_node);
        } else {
            ui.set_current_card(CardNode {
                question: "No cards due!".into(),
                answer: "".into(),
            });
        }
    });

    ui.set_due_total(deck_tree.review_count as i32);

    let deck_model = flatten_tree(&deck_tree);
    ui.set_deck_tree(deck_model.into());

    let _ = ui.run();
}
