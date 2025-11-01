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

pub fn remove_double_brackets(input: &str) -> String {
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
