use substring::Substring;
use tower_lsp::lsp_types::Position;
use typst_syntax::{SyntaxKind, SyntaxNode};
use typst_syntax::Source;
use std::ops::Range;
use crate::parse::{self, Document};

impl Document {
    pub fn new(in_str :&String) -> Self {
        let typst_source = Source::detached(in_str);
        let typst_root_node  = typst_source.root();
        let dirty_ranges = parse_recursive(&typst_source, in_str, typst_root_node);
        let clean_ranges = cleanup_range(dirty_ranges);
        Document {
            typst_source,
            text_chunks: clean_ranges,
        }
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
    pub fn get_chunk(&self, pos :&tower_lsp::lsp_types::Position) -> Option<String> {
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
    fn chunk_by_byte_index(&self, byte_index :usize) -> Option<Range<usize>> {
        for r in &self.text_chunks {
            if r.start <= byte_index && byte_index <= r.end {
                return Some(r.clone());
            }
        }
        None
    }
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
