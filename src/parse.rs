use std::{char, usize};
use dashmap::DashMap;
use tower_lsp::{lsp_types::{Position, TextDocumentItem, Url}, Client};
use typst_syntax::Source;
use std::ops::Range;

pub struct Backend {
    pub client: Client,
    pub document_map: DashMap<Url, Document>,
}
pub struct Document {
    pub typst_source: Source,
    pub text_chunks: Vec<Range<usize>>,
    pub diagnostics: Vec<Diagnostic>,
    pub diagnostics_lsp: Vec<tower_lsp::lsp_types::Diagnostic>,
}
pub struct Diagnostic {
    pub range :Range<usize>,
    pub source :DiagnosticSource,
}
pub enum DiagnosticSource {
    LanguageTool(crate::language_tool::LTDiagnostic)
}

impl Backend {
    pub fn create_document(&self, uri :&Url, text :&String) {
        self.document_map.insert(uri.clone(), Document::new(text));
    }

}
// Finds and returns the word a position pos. If none is found, None is returned
pub fn find_word(doc :String, pos :Position) -> Option<String> {
    let char_num :usize = match pos.character.try_into() {
        Ok(c) => {c},
        Err(_) => {return None},
    };
    let line = match find_line( doc, pos) {
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
fn find_line(doc :String, pos :Position) -> Option<String> {
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
pub async fn mark_text(uri :&Url, working_doc :&Document, client :&Client) {
    let mut diagnostics :Vec<tower_lsp::lsp_types::Diagnostic> = vec![];
    for a in &working_doc.text_chunks {
        let b = working_doc.typst_source.get(a.clone()).unwrap();
        let (line1, character1) = working_doc.range_to_line_character(a.start).unwrap();
        let (line2, character2) = working_doc.range_to_line_character(a.end).unwrap();
        let r = tower_lsp::lsp_types::Range {
            start: Position {
                line: line1 as u32,
                character: character1 as u32,
            },
            end: Position {
                line: line2 as u32,
                character: character2 as u32,
            }
        };

        //let message = format!("startLine: {}, startChar: {} | endLine: {}, endChar: {} ", c.range.start.line, c.range.start.character, c.range.end.line, c.range.end.character);
        let message = format!("startLine: {}, startChar: {} | endLine: {}, endChar: {} | content: \"{}\"", 
                              r.start.line, 
                              r.start.character, 
                              r.end.line, 
                              r.end.character, 
                              b
                              );
        diagnostics.push(
            tower_lsp::lsp_types::Diagnostic {
                range: r,
                severity: None,
                code: None,
                code_description: None,
                source: None,
                message,
                related_information: None,
                tags: None,
                data: None
            }
            );
    }
    client.publish_diagnostics(uri.clone(), diagnostics, None).await;
}
