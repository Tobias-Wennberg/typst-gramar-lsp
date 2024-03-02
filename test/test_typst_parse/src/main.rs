mod parse;
mod typst_parse;
mod language_tool;
use substring::Substring;
use tower_lsp::lsp_types::Position;
use typst_syntax::{SyntaxKind, SyntaxNode};
use typst_syntax::Source;
use std::ops::Range;
use std::fs::File;
use std::io::Read;

fn printRanges(ranges :&Vec<Range<usize>>) {
    for (i,r) in ranges.iter().enumerate() {
        println!("index: {}, Num: {}, Start: {}, End: {}", i, i, r.start, r.end);
    }
}
#[tokio::main]
async fn main() {
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
    let parsed = parse::Document::new(&content);
    let lt = language_tool::run_diagnostic(&parsed, &parsed.text_chunks).await.unwrap();
    for i in &lt.1 {
        println!("Start line: {}, column: {}, End line: {}, column: {} | message: {}",
                 i.range.start.line,
                 i.range.start.character,
                 i.range.end.line,
                 i.range.end.character,
                 i.message);
    }
    println!("lt 1 length: {}, lt 2 length: {}", lt.0.len(), lt.1.len());
}
