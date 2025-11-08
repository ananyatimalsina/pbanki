use std::rc::Rc;

use anki_proto::decks::DeckTreeNode;

use slint::Model;

use crate::DeckNode;

pub fn flatten_tree(node: &DeckTreeNode) -> Rc<slint::VecModel<DeckNode>> {
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

pub fn paginate_text(text: &str, chars_per_page: usize) -> Vec<slint::SharedString> {
    if text.is_empty() {
        return vec![slint::SharedString::new()];
    }

    let estimated_pages = (text.len() / chars_per_page).max(1);
    let mut pages = Vec::with_capacity(estimated_pages + 1);
    let mut current_page = String::with_capacity(chars_per_page);
    let mut current_len = 0;
    let mut first_word_in_page = true;

    for word in text.split_whitespace() {
        let word_len = word.chars().count();
        let space_needed = if first_word_in_page { 0 } else { 1 };

        if current_len + word_len + space_needed > chars_per_page && !current_page.is_empty() {
            pages.push(current_page.as_str().into());
            current_page.clear();
            current_len = 0;
            first_word_in_page = true;
        }

        if !first_word_in_page {
            current_page.push(' ');
            current_len += 1;
        }
        current_page.push_str(word);
        current_len += word_len;
        first_word_in_page = false;
    }

    if !current_page.is_empty() {
        pages.push(current_page.as_str().into());
    }

    if pages.is_empty() {
        pages.push(slint::SharedString::new());
    }

    pages
}

pub fn strip_html_remove_brackets_and_paginate(
    html: &str,
    remove_brackets: bool,
    chars_per_page: usize,
) -> Vec<slint::SharedString> {
    let stripped = anki::text::strip_html(html);
    let text = stripped.as_ref();

    if text.is_empty() {
        return vec![slint::SharedString::new()];
    }

    let estimated_pages = (text.len() / chars_per_page).max(1);
    let mut pages = Vec::with_capacity(estimated_pages + 1);
    let mut current_page = String::with_capacity(chars_per_page);
    let mut current_len = 0;
    let mut first_word_in_page = true;

    let mut chars = text.chars().peekable();
    let mut word_buffer = String::with_capacity(128);
    let mut skipping_brackets = false;

    while let Some(c) = chars.next() {
        if remove_brackets {
            if !skipping_brackets {
                if c == '[' {
                    if let Some(&'[') = chars.peek() {
                        chars.next();
                        skipping_brackets = true;
                        continue;
                    }
                }
            } else {
                if c == ']' {
                    if let Some(&']') = chars.peek() {
                        chars.next();
                        skipping_brackets = false;
                    }
                }
                continue;
            }
        }

        if c.is_whitespace() {
            if !word_buffer.is_empty() {
                let word_len = word_buffer.chars().count();
                let space_needed = if first_word_in_page { 0 } else { 1 };

                if current_len + word_len + space_needed > chars_per_page
                    && !current_page.is_empty()
                {
                    pages.push(current_page.as_str().into());
                    current_page.clear();
                    current_len = 0;
                    first_word_in_page = true;
                }

                if !first_word_in_page {
                    current_page.push(' ');
                    current_len += 1;
                }
                current_page.push_str(&word_buffer);
                current_len += word_len;
                first_word_in_page = false;
                word_buffer.clear();
            }
        } else {
            word_buffer.push(c);
        }
    }

    if !word_buffer.is_empty() {
        let word_len = word_buffer.chars().count();
        let space_needed = if first_word_in_page { 0 } else { 1 };

        if current_len + word_len + space_needed > chars_per_page && !current_page.is_empty() {
            pages.push(current_page.as_str().into());
            current_page.clear();
            first_word_in_page = true;
        }

        if !first_word_in_page {
            current_page.push(' ');
        }
        current_page.push_str(&word_buffer);
    }

    if !current_page.is_empty() {
        pages.push(current_page.as_str().into());
    }

    if pages.is_empty() {
        pages.push(slint::SharedString::new());
    }

    pages
}
