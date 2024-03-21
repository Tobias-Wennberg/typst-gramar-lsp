use serde::{Deserialize, Serialize};
use tower_lsp::lsp_types::{Position, Range};
use std::str::Chars;
use crate::components::{self, Diagnostic};
use crate::components::language_tool::LTDiagnostic;
use std::{
    collections::HashMap, 
    error::Error, 
    fs::File, 
    io::BufReader,
	collections::HashSet,
};
use languagetool_rust::{
    check::{
        DataAnnotation,
        CheckRequest,
        Data,
    },

    server::ServerClient,
    CheckResponse
};
use crate::parse;
use typst_syntax::SyntaxNode;
use typst_syntax::SyntaxKind;

pub async fn check(
    document :&parse::Document,
    ) -> (Vec<Diagnostic>, Vec<tower_lsp::lsp_types::Diagnostic>) {
    let typst_text :String = document.typst_source.text().to_string();
	let client = ServerClient::new("http://127.0.0.1", "8081");
    let typst_nodes = typst_syntax::parse(&typst_text);
    let converted_nodes = convert(&typst_nodes, &Rules::new(), 10000);

    let mut position :PositionLogic = PositionLogic::new(&typst_text);

    let mut out :(Vec<Diagnostic>, Vec<tower_lsp::lsp_types::Diagnostic>) =(vec!{}, vec!{}); 
	for items in converted_nodes {
		let req = CheckRequest::default()
			.with_language("auto".to_string())
			.with_data(Data::from_iter(items.0));

		let response = &mut client.check(&req).await.unwrap();
        filter_response(response, &Default::default());

        let mut out_pulls = add_chunk(document, &mut position, response, items.1);
        out.0.append(&mut out_pulls.0);
        out.1.append(&mut out_pulls.1);
	}

    out
}
fn add_chunk(
    document :&parse::Document,
    start : &mut PositionLogic, 
    response :&CheckResponse,
    total: usize,
    ) -> (Vec<Diagnostic>, Vec<tower_lsp::lsp_types::Diagnostic>){
    let mut last = 0;
    let mut out :(Vec<Diagnostic>, Vec<tower_lsp::lsp_types::Diagnostic>) =(vec!{}, vec!{}); 
	for info in &response.matches {
		start.advance(info.offset - last);
		let mut end = start.clone();
		end.advance(info.length);
        let start_line :u32 = start.line as u32;
        let start_column :u32 = start.column as u32;
        let end_line :u32 = end.line as u32;
        let end_column :u32 = end.column as u32;

        let r = Range {
            start: Position {
                    line: start_line,
                    character: start_column
                },
            end: Position {
                line: end_line,
                character: end_column
            },
        };

        let towe_lsp_val = tower_lsp::lsp_types::Diagnostic {
            range: r,
            severity: None,
            code: None,
            code_description: None,
            message: info.message.clone(),
            related_information: None,
            tags: None,
            data: None,
            source: None
        };
        out.1.push(
            towe_lsp_val.clone()
        );
        let typst_range :std::ops::Range<usize> = std::ops::Range {
            start: document.typst_source.line_column_to_byte(r.start.line as usize, r.start.character as usize).unwrap(),
            end: document.typst_source.line_column_to_byte(r.end.line as usize, r.end.character as usize).unwrap(),
        };
            
        out.0.push(
            Diagnostic {
                range: typst_range,
                diagnostics_lsp: towe_lsp_val.clone(),
                source_data: components::DiagnosticSourceData::LanguageTool(LTDiagnostic {
                    replacements: info.replacements.clone(),
                    rule: info.rule.clone()
                }),
                source: components::DiagnosticSource::LanguageTool,
        });
		last = info.offset;
	}
	start.advance(total - last);

    out
}
fn filter_response(response: &mut CheckResponse, dict: &HashSet<String>) {
	for m in std::mem::take(&mut response.matches).into_iter() {
		// Only handle misspellings
		if m.rule.issue_type.as_str() != "misspelling" {
			response.matches.push(m);
			continue;
		}
		// Check if the word is contained in the dictionary
		let ctx = &m.context;
		let mut chars = ctx.text.char_indices();
		let start = chars.nth(ctx.offset).map_or(0, |(idx, _)| idx);
		let end = chars
			.nth(ctx.length.wrapping_sub(1))
			.map_or(ctx.text.len(), |(idx, _)| idx);
		let word = &ctx.text[start..end];
		if dict.contains(word) {
			continue;
		}
		response.matches.push(m);
	}
}
fn convert(
	node: &SyntaxNode,
	rules: &Rules,
	max_length: usize,
) -> Vec<(Vec<DataAnnotation>, usize)> {
	let state = State { mode: Mode::Markdown };
	let mut output = Output::new();
	for child in node.children() {
		state.convert(child, &mut output, rules);
		if child.kind() == SyntaxKind::Parbreak {
			output.maybe_seperate(max_length);
		}
	}
	output.result()
}
#[derive(Serialize, Deserialize)]
struct Rules {
	functions: HashMap<String, Function>,
}

#[derive(Serialize, Deserialize)]
struct Function {
	before: String,
	after: String,
}

impl Rules {
	fn new() -> Self {
		Self { functions: HashMap::new() }
	}

	fn load(path: &String) -> Result<Self, Box<dyn Error>> {
		let file = File::open(path)?;
		let reader = BufReader::new(file);
		let rules = serde_json::from_reader(reader)?;
		Ok(rules)
	}
}
enum OutputState {
	Text(String),
	Markup(String),
	Encoded(String, String),
}

struct Output {
	items: Vec<(Vec<DataAnnotation>, usize)>,
	state: OutputState,
}

impl Output {
	fn new() -> Self {
		Self {
			items: vec![(Vec::new(), 0)],
			state: OutputState::Text(String::new()),
		}
	}

	fn add_item(&mut self, item: DataAnnotation) {
		if let Some(text) = &item.text {
			self.items.last_mut().unwrap().1 += text.chars().count();
		}
		if let Some(text) = &item.markup {
			self.items.last_mut().unwrap().1 += text.chars().count();
		}
		self.items.last_mut().unwrap().0.push(item);
	}

	// is possible without cloning, but not naive in safe rust
	fn add_text(&mut self, text: String) {
		self.state = match &self.state {
			OutputState::Text(t) => OutputState::Text(t.clone() + &text),
			OutputState::Markup(t) => {
				self.add_item(DataAnnotation::new_markup(t.clone()));
				OutputState::Text(text)
			},
			OutputState::Encoded(t, a) => {
				self.add_item(DataAnnotation::new_interpreted_markup(t.clone(), a.clone()));
				OutputState::Text(text)
			},
		}
	}

	fn add_markup(&mut self, text: String) {
		self.state = match &self.state {
			OutputState::Text(t) => {
				self.add_item(DataAnnotation::new_text(t.clone()));
				OutputState::Markup(text)
			},
			OutputState::Markup(t) => OutputState::Markup(t.clone() + &text),
			OutputState::Encoded(t, a) => {
				self.add_item(DataAnnotation::new_interpreted_markup(t.clone(), a.clone()));
				OutputState::Markup(text)
			},
		}
	}
	fn add_encoded(&mut self, text: String, res: String) {
		self.state = match &self.state {
			OutputState::Text(t) => {
				self.add_item(DataAnnotation::new_text(t.clone()));
				OutputState::Encoded(text, res)
			},
			OutputState::Markup(t) => {
				self.add_item(DataAnnotation::new_markup(t.clone()));
				OutputState::Encoded(text, res)
			},
			OutputState::Encoded(t, a) => OutputState::Encoded(t.clone() + &text, a.clone() + &res),
		}
	}

	fn flush(&mut self) {
		match &self.state {
			OutputState::Text(t) => self.add_item(DataAnnotation::new_text(t.clone())),
			OutputState::Markup(t) => self.add_item(DataAnnotation::new_markup(t.clone())),
			OutputState::Encoded(t, a) => {
				self.add_item(DataAnnotation::new_interpreted_markup(t.clone(), a.clone()));
			},
		}
	}

	fn maybe_seperate(&mut self, max: usize) {
		if self.items.last().unwrap().1 > max {
			self.flush();
			self.state = OutputState::Text(String::new());
			self.items.push((Vec::new(), 0));
		}
	}

	fn result(mut self) -> Vec<(Vec<DataAnnotation>, usize)> {
		self.flush();
		self.items
	}
}

#[derive(PartialEq, Clone, Copy)]
enum Mode {
	Markdown,
	Code,
}

#[derive(Clone, Copy)]
struct State {
	mode: Mode,
}

impl State {
	fn convert(mut self, node: &SyntaxNode, output: &mut Output, rules: &Rules) {
		match node.kind() {
			SyntaxKind::Text if self.mode == Mode::Markdown => output.add_text(node.text().into()),
			SyntaxKind::Equation => {
				output.add_encoded(node.text().into(), String::from("0"));
				Self::skip(node, output);
			},
			SyntaxKind::FuncCall => {
				self.mode = Mode::Code;
				let name = node.children().next().unwrap().text();
				let rule = rules.functions.get(name.as_str());
				if let Some(f) = rule {
					output.add_encoded(String::new(), f.before.to_owned());
				}
				for child in node.children() {
					self.convert(child, output, rules);
				}
				if let Some(f) = rule {
					output.add_encoded(String::new(), f.after.to_owned());
				}
			},
			SyntaxKind::Code
			| SyntaxKind::ModuleImport
			| SyntaxKind::ModuleInclude
			| SyntaxKind::LetBinding
			| SyntaxKind::ShowRule
			| SyntaxKind::SetRule => {
				self.mode = Mode::Code;
				for child in node.children() {
					self.convert(child, output, rules);
				}
			},
			SyntaxKind::Heading => {
				output.add_encoded(String::new(), String::from("\n\n"));
				for child in node.children() {
					self.convert(child, output, rules);
				}
				output.add_encoded(String::new(), String::from("\n\n"));
			},
			SyntaxKind::Ref => {
				output.add_encoded(String::new(), String::from("X"));
				Self::skip(node, output);
			},
			SyntaxKind::LeftBracket | SyntaxKind::RightBracket => {
				output.add_encoded(node.text().into(), String::from("\n\n"));
			},
			SyntaxKind::Markup => {
				self.mode = Mode::Markdown;
				for child in node.children() {
					self.convert(child, output, rules);
				}
			},
			SyntaxKind::Shorthand if node.text() == "~" => {
				output.add_encoded(node.text().into(), String::from(" "));
			},
			SyntaxKind::Space if self.mode == Mode::Markdown => output.add_text(node.text().into()),
			SyntaxKind::Parbreak => output.add_encoded(node.text().into(), String::from("\n\n")),
			SyntaxKind::SmartQuote if self.mode == Mode::Markdown => {
				output.add_text(node.text().into())
			},
			_ => {
				output.add_markup(node.text().into());
				for child in node.children() {
					self.convert(child, output, rules);
				}
			},
		}
	}

	fn skip(node: &SyntaxNode, output: &mut Output) {
		output.add_markup(node.text().into());
		for child in node.children() {
			Self::skip(child, output);
		}
	}
}
pub struct PositionLogic<'a> {
	line: usize,
	column: usize,
	content: Chars<'a>,
}


impl<'a> PositionLogic<'a> {
	pub fn new(content: &'a str) -> Self {
		Self {
			line: 0,
			column: 0,
			content: content.chars(),
		}
	}


	fn advance(&mut self, amount: usize) {
		for _ in 0..amount {
			match self.content.next().unwrap() {
				'\n' => {
					self.line += 1;
					self.column = 0;
				},
				_ => {
					self.column += 1;
				},
			}
		}
	}
    fn clone(&self) -> Self {
        PositionLogic {
            line: self.line.clone(),
            column: self.column.clone(),
            content: self.content.clone(),
        }
    }
}
