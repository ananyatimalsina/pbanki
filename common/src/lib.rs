slint::include_modules!();

pub mod api;
pub mod utils;

pub use api::{LearnSession, next_card, rate_card, update_deck_tree};
