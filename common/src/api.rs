use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use anki::scheduler::answering::CardAnswer;
use anki::scheduler::states::SchedulingStates;
use anki::timestamp::{TimestampMillis, TimestampSecs};
use anki::{collection::CollectionBuilder, prelude::I18n};

use crate::{CardNode, DeckNode, DeckTree};

use slint::ModelRc;

pub struct LearnSession {
    pub collection: Rc<RefCell<anki::collection::Collection>>,
    pub current_card: RefCell<CardNode>,
    pub states: RefCell<Option<SchedulingStates>>,
    pub start_time: RefCell<Option<Instant>>,
}

pub fn init_session(collection_path: &str) -> Rc<LearnSession> {
    let mut col = match CollectionBuilder::new(format!("{}/collection.anki2", collection_path))
        .set_media_paths(
            format!("{}/collection.media/", collection_path),
            format!("{}/collection.media.db2", collection_path),
        )
        .set_tr(I18n::new(&["en"]))
        .build()
    {
        Ok(col) => col,
        Err(e) => {
            eprintln!("Failed to open collection: {:?}", e);
            panic!("Cannot continue without a valid collection.");
        }
    };

    let search = "note:\"Image Occlusion Enhanced\"";

    if let Ok(card_ids) = col.search_cards(search, anki::search::SortMode::NoOrder) {
        let _ = col.bury_or_suspend_cards(
            &card_ids,
            anki_proto::scheduler::bury_or_suspend_cards_request::Mode::Suspend,
        );
    }

    Rc::new(LearnSession {
        collection: Rc::new(RefCell::new(col)),
        current_card: RefCell::new(CardNode::default()),
        states: RefCell::new(None),
        start_time: RefCell::new(None),
    })
}

pub fn update_deck_tree(session: &LearnSession) -> DeckTree {
    let deck_tree = session
        .collection
        .borrow_mut()
        .deck_tree(Some(TimestampSecs::now()))
        .unwrap();

    let deck_nodes = crate::utils::flatten_tree(&deck_tree);

    DeckTree {
        due_total: deck_tree.review_count as i32,
        deck_nodes: deck_nodes.into(),
    }
}

pub fn next_card(session: &LearnSession, deck: DeckNode, chars_per_page: i32) -> CardNode {
    let mut col_borrow = session.collection.borrow_mut();
    let _ = col_borrow.set_current_deck(anki::decks::DeckId(deck.id));
    let queued_cards = col_borrow.get_queued_cards(1, false).unwrap();
    drop(col_borrow);
    let queued_card = queued_cards.cards.first();

    if let Some(card) = queued_card {
        *session.start_time.borrow_mut() = Some(Instant::now());
        *session.states.borrow_mut() = Some(card.states.clone());

        let durations = session
            .collection
            .borrow_mut()
            .describe_next_states(&card.states)
            .unwrap_or_else(|_| vec!["".into(); 4]);

        let rendered = session
            .collection
            .borrow_mut()
            .render_existing_card(card.card.id(), false, false)
            .unwrap();

        let mut answer = anki::text::strip_html(rendered.answer().as_ref()).into_owned();

        // Handle special case for type in cards
        if answer.contains("[[type:") {
            let note_opt = session
                .collection
                .borrow_mut()
                .storage
                .get_note(card.card.note_id())
                .unwrap();

            if let Some(note) = note_opt {
                let fields = note.fields();
                let notetype_id = note.notetype_id;
                let notetype = session
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
        let updated_deck = DeckNode {
            new: queued_cards.new_count as i32,
            learn: queued_cards.learning_count as i32,
            due: queued_cards.review_count as i32,
            ..deck
        };
        let card_node = CardNode {
            id: card.card.id().0,
            deck: updated_deck,
            question: Rc::new(slint::VecModel::from(
                crate::utils::strip_html_remove_brackets_and_paginate(
                    rendered.question().as_ref(),
                    true,
                    chars_per_page as usize,
                )
                .into_iter()
                .map(|s| s.into())
                .collect::<Vec<slint::SharedString>>(),
            ))
            .into(),
            answer: Rc::new(slint::VecModel::from(
                crate::utils::paginate_text(&answer, chars_per_page as usize)
                    .into_iter()
                    .map(|s| s.into())
                    .collect::<Vec<slint::SharedString>>(),
            ))
            .into(),
            durations: Rc::new(slint::VecModel::from(
                durations
                    .into_iter()
                    .map(|s| s.into())
                    .collect::<Vec<slint::SharedString>>(),
            ))
            .into(),
        };

        *session.current_card.borrow_mut() = card_node.clone();
        card_node
    } else {
        CardNode {
            id: -1,
            deck,
            question: ModelRc::new(slint::VecModel::from(vec!["No more cards due!".into()])),
            answer: ModelRc::new(slint::VecModel::default()),
            durations: ModelRc::new(slint::VecModel::default()),
        }
    }
}

pub fn rate_card(
    session: &LearnSession,
    rating: i32,
    deck: DeckNode,
    chars_per_page: i32,
) -> CardNode {
    let card = session.current_card.borrow().clone();
    let states = session.states.borrow().clone();

    if card.id == -1 || states.is_none() {
        return CardNode {
            id: -1,
            deck,
            question: ModelRc::new(slint::VecModel::from(vec!["No more cards due!".into()])),
            answer: ModelRc::new(slint::VecModel::default()),
            durations: ModelRc::new(slint::VecModel::default()),
        };
    }

    let states = states.unwrap();
    let elapsed = session
        .start_time
        .borrow()
        .map(|t| t.elapsed().as_millis() as u32)
        .unwrap_or(0);

    let (new_state, rating_enum) = match rating {
        0 => (
            states.again.clone(),
            anki::scheduler::answering::Rating::Again,
        ),
        1 => (
            states.hard.clone(),
            anki::scheduler::answering::Rating::Hard,
        ),
        2 => (
            states.good.clone(),
            anki::scheduler::answering::Rating::Good,
        ),
        3 => (
            states.easy.clone(),
            anki::scheduler::answering::Rating::Easy,
        ),
        _ => {
            return CardNode {
                id: -1,
                deck,
                question: ModelRc::new(slint::VecModel::from(vec!["No more cards due!".into()])),
                answer: ModelRc::new(slint::VecModel::default()),
                durations: ModelRc::new(slint::VecModel::default()),
            };
        }
    };

    let mut answer = CardAnswer {
        card_id: anki::card::CardId(card.id),
        current_state: states.current.clone(),
        new_state,
        rating: rating_enum,
        answered_at: TimestampMillis::now(),
        milliseconds_taken: elapsed,
        custom_data: None,
        from_queue: true,
    };

    let _ = session.collection.borrow_mut().answer_card(&mut answer);

    next_card(session, deck, chars_per_page)
}
