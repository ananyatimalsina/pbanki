use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::time::Instant;

use anki::scheduler::answering::CardAnswer;
use anki::scheduler::states::SchedulingStates;
use anki::timestamp::{TimestampMillis, TimestampSecs};
use anki::{collection::CollectionBuilder, prelude::I18n};

use crate::{CardNode, DeckNode, DeckTree, SyncManager, SyncResult, SyncStatus, Translations};

use slint::ModelRc;

pub struct LearnSession {
    pub collection: Rc<RefCell<anki::collection::Collection>>,
    pub current_card: RefCell<Option<i64>>,
    pub states: RefCell<Option<SchedulingStates>>,
    pub start_time: RefCell<Option<Instant>>,
    pub sync_manager: Rc<SyncManager>,
}

pub fn init_session(config: &crate::config::Config) -> Rc<LearnSession> {
    let collection_path = &config.general.collection_path;
    let language = &config.general.language;

    let col_file = format!("{}/collection.anki2", collection_path);

    if let Some(parent) = Path::new(&col_file).parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("Failed to create collection directory: {:?}", e);
            panic!(
                "Cannot create collection directory at: {}",
                parent.display()
            );
        }
    }

    let mut col = match CollectionBuilder::new(&col_file)
        .set_tr(I18n::new(&[language.as_str()]))
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
        current_card: RefCell::new(None),
        states: RefCell::new(None),
        start_time: RefCell::new(None),
        sync_manager: Rc::new(SyncManager::new()),
    })
}

pub fn init_translations(session: &LearnSession) -> Translations {
    let col_borrow = session.collection.borrow();
    let i181 = col_borrow.tr();

    Translations {
        show_answer: i181.studying_show_answer().as_ref().into(),
        again: i181.studying_again().as_ref().into(),
        hard: i181.studying_hard().as_ref().into(),
        good: i181.studying_good().as_ref().into(),
        easy: i181.studying_easy().as_ref().into(),
        cards_due_suffix: {
            let full = i181.statistics_cards_due(0);
            full.as_ref().replace("0", "").into()
        },
        no_cards_due: i181.studying_no_cards_are_due_yet().as_ref().into(),
    }
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
    let queued_card = queued_cards.cards.first();

    if let Some(card) = queued_card {
        *session.start_time.borrow_mut() = Some(Instant::now());
        *session.states.borrow_mut() = Some(card.states.clone());

        let durations = col_borrow
            .describe_next_states(&card.states)
            .unwrap_or_else(|_| vec!["".into(); 4]);

        let rendered = col_borrow
            .render_existing_card(card.card.id(), false, false)
            .unwrap();

        let mut answer = anki::text::strip_html(rendered.answer().as_ref()).into_owned();

        // Handle special case for type in cards
        if answer.contains("[[type:") {
            let note_opt = col_borrow.storage.get_note(card.card.note_id()).unwrap();

            if let Some(note) = note_opt {
                let fields = note.fields();
                let notetype_id = note.notetype_id;
                let notetype = col_borrow.get_notetype(notetype_id).unwrap();

                if let Some(notetype) = notetype {
                    for (index, field) in notetype.fields.iter().enumerate() {
                        if field.name == "Back" {
                            answer = (&fields[index]).to_owned();
                        }
                    }
                }
            }
        }
        drop(col_borrow);
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
                ),
            ))
            .into(),
            answer: Rc::new(slint::VecModel::from(crate::utils::paginate_text(
                &answer,
                chars_per_page as usize,
            )))
            .into(),
            durations: Rc::new(slint::VecModel::from(
                durations
                    .into_iter()
                    .map(|s| s.into())
                    .collect::<Vec<slint::SharedString>>(),
            ))
            .into(),
        };

        *session.current_card.borrow_mut() = card_node.id.into();
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
    let card_id = session.current_card.borrow().unwrap_or(-1);
    let states = session.states.borrow().clone();

    if card_id == -1 || states.is_none() {
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
        card_id: anki::card::CardId(card_id),
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

pub fn sync_ankiweb(session: &LearnSession, config: &crate::config::Config) -> SyncResult {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let ankiweb_config = &config.ankiweb;

    let hkey = if let Some(token) = &ankiweb_config.token {
        token.clone()
    } else {
        match rt.block_on(
            session
                .sync_manager
                .login(&ankiweb_config.username, &ankiweb_config.password),
        ) {
            Ok(token) => {
                let mut cfg = config.clone();
                cfg.ankiweb.token = Some(token.clone());
                let _ = cfg.save();
                token
            }
            Err(e) => {
                return SyncResult {
                    success: false,
                    message: format!("Login failed: {}", e),
                    server_message: None,
                };
            }
        }
    };

    let result = rt.block_on(async {
        let mut col = session.collection.borrow_mut();
        session.sync_manager.sync_collection(&mut col, &hkey).await
    });

    match result {
        Ok(sync_result) => sync_result,
        Err(e) => SyncResult {
            success: false,
            message: format!("Sync failed: {}", e),
            server_message: None,
        },
    }
}

pub fn get_sync_status(session: &LearnSession) -> SyncStatus {
    session.sync_manager.get_status()
}
