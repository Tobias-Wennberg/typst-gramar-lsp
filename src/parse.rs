use std::{char, ops::Index, u32, usize};

use dashmap::DashMap;
use im_rc::HashMap;
use tower_lsp::{lsp_types::{Position, TextDocumentItem, Url}, Client};
use ropey::Rope;


pub struct Backend {
    pub client: Client,
    pub document_map: DashMap<String, Document>,
}
pub struct Document {
    pub body_rope: Rope,
    pub text_document_item: TextDocumentItem,
}

impl Backend {
    pub fn create_document(&self, doc :&TextDocumentItem) {
        let rope = ropey::Rope::from_str(&doc.text);
        self.document_map.insert(doc.uri.to_string(), Document {
            body_rope: rope,
            text_document_item: doc.clone(),
        });
    }

    pub fn on_change(&self, working_doc :&Document, doc_changes: &TextDocumentItem) {
        let rope = ropey::Rope::from_str(&doc_changes.text);
        self.document_map.insert(doc_changes.uri.to_string(), Document {
            body_rope: rope,
            text_document_item: doc_changes.clone(),
        });
    }
}
// Finds and returns the word a position pos. If none is found, None is returned
pub fn find_word_str(doc :String, pos :Position) -> Option<String> {
    let char_num :usize = match pos.character.try_into() {
        Ok(c) => {c},
        Err(_) => {return None},
    };
    let line = match find_line_str( doc, pos) {
        Some(c) => {c},
        None => {"".to_string()},
    }.to_string();
    let line_chars = line.as_bytes();
    if line_chars.len() <= char_num {
        return None
    }
    if !(line_chars[char_num] as char).is_alphabetic() {
        return None 
    }
    let mut start = "".to_string();
    let end :String;
    if char_num >= 1 {
        if (line_chars[char_num-1] as char).is_alphabetic() {
            start = match line[..char_num].split_whitespace().last() {
                Some(c) => {c.to_string()},
                None => {"".to_string()},
            };
        }
    }
    if char_num+1 < line_chars.len() && (line_chars[char_num+1] as char).is_alphabetic() {
        end = match line[char_num..].split_whitespace().next() {
            Some(c) => {c.to_string()},
            None => {
                (line_chars[char_num] as char).to_string()
            },
        };
    } else {
        end = (line_chars[char_num] as char).to_string()
    }
    let mut word = start+&end;
    word = word.chars().filter(|&c| c.is_alphabetic()).collect();
    if word == "" {
        return None;
    }
    return Some(word);
}
fn find_line_str(doc :String, pos :Position) -> Option<String> {
    let line_num :usize = match pos.line.try_into() {
        Ok(c) => {c},
        Err(_) => {return None},
    };
    let lines :Vec<&str> = doc.lines().collect();
    let line = match lines.get(line_num) {
        Some(c) => {c},
        None => {return None},
    }.to_string();
    return Some(line);
}
/*
pub fn find_word_rope(doc :Rope, pos :Position) -> Option<String> {
    let line_num = pos.line.try_into().unwrap();
    let char_num = pos.line.try_into().unwrap();
    if line_num > doc.len_lines() {
        return None;
    }
    let mut start = 0;
    let mut end = 0;
    let line_text = doc.line(line_num);
    line_text.
    while start > 0 && !line_text.is_char_boundary(start - 1) {
        start -= 1;
    }

    // Find the end of the word
    while end < line_text.len_chars() && !line_text.is_char_boundary(end) {
        end += 1;
    }

    // Extract the word from the Rope
    let word = line_text.slice(start..end).to_string();
    return None;

}
*/
