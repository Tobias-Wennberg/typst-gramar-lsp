use substring::Substring;
use tower_lsp::lsp_types::Position;
use typst_syntax::{SyntaxKind, SyntaxNode};
use typst_syntax::Source;
use std::ops::Range;
use std::fs::File;
use std::io::Read;

fn printRanges(ranges :&Vec<Range<usize>>) {
    for (i,r) in ranges.iter().enumerate() {
        println!("Num: {}, Start: {}, End: {}", i, i, r.start, r.end);
    }
}

pub struct Chunk {
    pub range: tower_lsp::lsp_types::Range,
    pub content: String
}
pub fn parse_file(in_str :&String) -> (Vec<Chunk>, Source) {
    let typst_source = Source::detached(in_str);
    let typst_root_node  = typst_source.root();
    let dirty_ranges = parse_recursive(&typst_source, in_str, typst_root_node);
    printRanges(&dirty_ranges);
    let clean_ranges = cleanup_range(dirty_ranges.clone());
    let chunks = ranges_to_chunks(in_str, &typst_source, &clean_ranges);
    /*
    chunks.push(
        Chunk {
            range: tower_lsp::lsp_types::Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: chunks.len() as u32,
                    character: before_clean as u32,
                }
            },
            content: "HEJ!".to_string()
        }

    );
    */


    (chunks, typst_source)
}

fn ranges_to_chunks(in_str :&String, source :&Source, ranges :&Vec<Range<usize>>) -> Vec<Chunk> {
    let mut chunks = Vec::with_capacity(ranges.len());
    for r in ranges {
        chunks.push( Chunk {
            range: tower_lsp::lsp_types::Range {
                start: Position {
                    line: source.byte_to_line(r.start).unwrap() as u32,
                    character: source.byte_to_column(r.start).unwrap() as u32
                },
                end: Position {
                    line: source.byte_to_line(r.end).unwrap() as u32,
                    character: source.byte_to_column(r.end).unwrap() as u32 +1
                }
            },
            content: in_str.substring(r.start,r.end).to_string(),
        }
            );
    }

    chunks
}

fn cleanup_range(mut ranges :Vec<Range<usize>>) -> Vec<Range<usize>> {
    ranges.sort_by_key(|range| range.start);
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

    if children.len() == 0 {
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
            SyntaxKind::Markup => true,
            SyntaxKind::Text => true,
            SyntaxKind::SmartQuote => true,
            SyntaxKind::Strong => true,
            SyntaxKind::Emph => true,
            SyntaxKind::Raw => true,
            SyntaxKind::Heading => true,
            SyntaxKind::HeadingMarker => true,
            SyntaxKind::ListItem => true,
            SyntaxKind::ListMarker => true,
            SyntaxKind::EnumItem => true,
            SyntaxKind::EnumMarker => true,
            SyntaxKind::TermItem => true,
            SyntaxKind::TermMarker => true,
            SyntaxKind::LineComment => false,
            SyntaxKind::BlockComment => true,
            _ => false,
    }

}
fn main() {
    // Replace "your_file.txt" with the actual path to your file
    let file_path = "555";

    // Attempt to open the file
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening file: {}", e);
            return;
        }
    };

    // Read the content of the file into a String
    let mut content = String::new();
    if let Err(e) = file.read_to_string(&mut content) {
        eprintln!("Error reading file: {}", e);
        return;
    }
    parse_file(&content);
}
