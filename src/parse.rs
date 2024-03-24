use std::{char, collections::{BTreeMap, HashMap}, usize};
use dashmap::DashMap;
use tower_lsp::{lsp_types::{Position, Url}, Client};
use typst_syntax::Source;
use std::ops::Range;
use typst_syntax::{SyntaxKind, SyntaxNode};

pub struct Backend {
    pub client: Client,
    pub document_map: DashMap<Url, Document>,
}
pub struct Document {
    pub typst_source: Source,
    pub text_chunks: Vec<Range<usize>>,
    pub diagnostics: Vec<crate::components::Diagnostic>,
    pub diagnostics_lsp: Vec<tower_lsp::lsp_types::Diagnostic>,
    //the source changes made, usefull for making the ranges stored up-to-date. The key is the
    //version of the document.
    pub source_change: BTreeMap<isize, Vec<SourceChange>>,
    pub latest_version: isize,
}
pub struct SourceChange {
    //The range of the data changed
    range: Range<usize>,
    //The enlargment of the data, may be negative and zero
    delta: isize,
}

impl Backend {
    pub fn create_document(&self, uri :&Url, version :isize, text :&String) {
        self.document_map.insert(uri.clone(), Document::new(version, text));
    }

}
impl Document {
    pub fn new(version :isize, in_str :&String) -> Self {
        let typst_source = Source::detached(in_str);
        let typst_root_node  = typst_source.root();
        let dirty_ranges = parse_recursive(&typst_source, in_str, typst_root_node);
        let clean_ranges = cleanup_range(dirty_ranges);
        Document {
            typst_source,
            text_chunks: clean_ranges,
            diagnostics_lsp: vec!{},
            diagnostics: vec!{},
            source_change: BTreeMap::new(),
            latest_version: version,
        }
    }
    pub fn change(&mut self, version :i32, change :&tower_lsp::lsp_types::TextDocumentContentChangeEvent) {
        self.latest_version = version as isize;
        let range = self.lsp_range_to_byte_range(&change.range.unwrap()).unwrap();
        self.typst_source.edit(range.clone(), &change.text);
        self.source_change
            .entry(version as isize)
            .or_insert(Vec::new()).push(SourceChange {
                range: range.clone(),
                delta: change.text.len() as isize - (range.end as isize - range.start as isize)
            });
    }
    // Corrects an old range to the changes, returns none if the range is out-of-bounds or changes
    // been made ower it
    pub fn correct_range(&self, version :isize, mut range :Range<usize>) -> Option<Range<usize>> {
        let valid_changes :Vec<&Vec<SourceChange>> = self.source_change 
            .range(version+1..)
            .map(|c| c.1)
            .collect();
        for some_changes in valid_changes {
            for change in some_changes {
                if change.range.end > self.typst_source.len_bytes() {
                    return None;
                }
                if change.range.start < range.end && range.start < change.range.end {
                    return None;
                }
                if range.start < change.range.start {
                    continue
                }
                range.start = (range.start as isize + change.delta) as usize;
                range.end = (range.end as isize + change.delta) as usize;
            }
        }
        Some(range)
    }
    //Return (line, character) or none if outside of source
    pub fn range_to_line_character(&self, r :usize) -> Option<(usize, usize)> {
        let l = match self.typst_source.byte_to_line(r) {
            Some(c) => c,
            None => return None,
        };
        let c = match self.typst_source.byte_to_column(r) {
            Some(c) => c,
            None => return None,
        };
        Some((l,c))
    }
    pub fn lsp_range_to_byte_range(&self, range :&tower_lsp::lsp_types::Range) -> Option<Range<usize>> {
        Some(Range{
            start: match self.typst_source.line_column_to_byte(range.start.line as usize, range.start.character as usize) {
                Some(c) => {c},
                None => {return None}
            },
            end: match self.typst_source.line_column_to_byte(range.end.line as usize, range.end.character as usize) {
                Some(c) => {c},
                None => {return None}
            },
        })
    }
    pub fn byte_range_to_lsp_range(&self, range :&Range<usize>) -> Option<tower_lsp::lsp_types::Range> {
        Some(
            tower_lsp::lsp_types::Range {
                start: tower_lsp::lsp_types::Position {
                    line: match self.typst_source.byte_to_line(range.start) {
                        Some(c) => {c as u32},
                        None => return None,
                    },
                    character: match self.typst_source.byte_to_column(range.start) {
                        Some(c) => {c as u32},
                        None => return None,
                    },
                },
                end: tower_lsp::lsp_types::Position {
                    line: match self.typst_source.byte_to_line(range.end) {
                        Some(c) => {c as u32},
                        None => return None,
                    },
                    character: match self.typst_source.byte_to_column(range.end) {
                        Some(c) => {c as u32},
                        None => return None,
                    },
                }
            }
        )

    }
    pub fn get_chunk_at_pos(&self, pos :&tower_lsp::lsp_types::Position) -> Option<String> {
        let byte_index = match self.typst_source.line_column_to_byte(pos.line as usize, pos.character as usize) {
            Some(c) => c,
            None => return None,
        };
        let range = match self.chunk_by_byte_index(byte_index) {
            Some(c) => c,
            None => return None,
        };

        match self.typst_source.get(range) {
            Some(c) => Some(c.to_string()),
            None => None
        }
    }
    pub fn get_chunk_by_range(&self, range :Range<usize>) -> Option<String> {
        match self.typst_source.get(range) {
            Some(c) => Some(c.to_string()),
            None => None
        }
    }
    fn chunk_by_byte_index(&self, byte_index :usize) -> Option<Range<usize>> {
        for r in &self.text_chunks {
            if r.start <= byte_index && byte_index <= r.end {
                return Some(r.clone());
            }
        }
        None
    }
    // Finds and returns the word a position pos. If none is found, None is returned
    pub fn find_word(&self, pos :Position) -> Option<String> {
        let char_num :usize = match pos.character.try_into() {
            Ok(c) => {c},
            Err(_) => {return None},
        };
        let line = match self.typst_source.get(
                    self.typst_source.line_to_range(pos.line as usize)?
                    ) {
            Some(c) => c,
            None => {return None;}
        };
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

        Some(word)
    }

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
fn cleanup_range(mut ranges :Vec<Range<usize>>) -> Vec<Range<usize>> {
    ranges.sort_by_key(|range| range.start);
    ranges = merge_range_neighbors(ranges);
    ranges.sort_by_key(|range| range.start);
    ranges = merge_range_neighbors(ranges);
    ranges.retain(|r| (r.end - r.start) > 2);

    ranges
}
fn merge_range_neighbors(ranges :Vec<Range<usize>>) -> Vec<Range<usize>> {
    let mut out_ranges :Vec<Range<usize>> = vec!{};
    if ranges.len() == 0 { return vec!{};}
    let mut working_range :Range<usize> = ranges[0].clone();
    for r in ranges {
        if !within(if r.start == 0 {0} else {r.start-1}, &working_range) {
            out_ranges.push(working_range);
            working_range = r.clone();
            continue
        }
        working_range.end = if r.end > working_range.end {r.end} else {working_range.end};

    }
    out_ranges

}
fn within(a :usize, b :&Range<usize>) -> bool {
    b.start <= a && a <= b.end
}

// Return a vec of the ranges that is text, needs cleaning
fn parse_recursive(typst_source :&Source, in_str :&String, this_node :&SyntaxNode) -> Vec<Range<usize>> {
    let mut chunks = vec!{};
    let children = this_node.children();
    let this_range = nod_to_range_unsafe(typst_source, this_node);
    let mut cursor = this_range.start;
    let this_is_text = is_text_kind(this_node);

    if children.len() == 0 && this_is_text {
        chunks.push(this_range);
    }

    for child in children {
        let child_range = nod_to_range_unsafe(typst_source, child);
        if this_is_text {
            chunks.push(Range{start: cursor, end: child_range.start});
        }
        cursor = child_range.end;
        chunks.append(&mut parse_recursive(typst_source, in_str, child));
    }
    chunks
}
fn nod_to_range_unsafe(source :&Source, node :&SyntaxNode)  -> Range<usize> {
    source.range(node.span()).unwrap()
}
fn is_text_kind(node :&SyntaxNode) -> bool {
    match node.kind() {
            SyntaxKind::Markup => false,
            SyntaxKind::Text => true,
            SyntaxKind::Space => false,
            SyntaxKind::Strong => true,
            SyntaxKind::Emph => true,
            SyntaxKind::Raw => true,
            SyntaxKind::Link => false,
            SyntaxKind::Heading => false,
            _ => false,
    }

}
