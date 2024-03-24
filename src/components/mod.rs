mod language_tool;
use std::ops::{Range, Deref};
use std::cmp::PartialEq;
use std::sync::Mutex;
use std::clone::Clone;

use dashmap::DashMap;
use lazy_static::lazy_static;
use tower_lsp::lsp_types::MessageType;

use crate::parse;

lazy_static! {
    static ref CODE_ACTIONS :DashMap<String, CodeActionSource> = DashMap::new();
    static ref DIAGNOSTICS :Mutex<Vec<Diagnostic>> = Mutex::new(Vec::new());
}

#[derive(Clone)]
pub struct Diagnostic {
   range :Range<usize>,
   version :isize,
   diagnostics_lsp :tower_lsp::lsp_types::Diagnostic,
   source_data :DiagnosticSourceData,
   source :DiagnosticSource,

}
impl Diagnostic {
    pub fn corrected_diagnostics_lsp(&self, document :&parse::Document) 
    -> Option<tower_lsp::lsp_types::Diagnostic> {
        let mut ret = self.diagnostics_lsp.clone();
        let corrected_range = match document.correct_range(self.version, self.range.clone()) {
            Some(c) => c ,
            None => {return None}
        };
        ret.range = match document.byte_range_to_lsp_range(&corrected_range) {
            Some(c) => {c},
            None => {return None},
        };
        Some(ret)
    }
}
#[derive(PartialEq, Clone)]
pub enum DiagnosticSource {
    LanguageTool,
}
#[derive(Clone)]
pub enum DiagnosticSourceData {
    LanguageTool(language_tool::LTDiagnostic),
    other,
}
pub enum CodeActionSource {
    LanguageToolCheckText(language_tool::LTCodeActionCheckText),
    LanguageToolRemoveDiagnostics(language_tool::LTCodeActionRemoveDiagnostic),
}

pub async fn send_diagnostics(client :&tower_lsp::Client, document :&crate::parse::Document, uri :&tower_lsp::lsp_types::Url) {
    client.publish_diagnostics(uri.clone(), get_lsp_diagnostics(document), None).await;
}

pub async fn code_actions(client :&tower_lsp::Client, document :&crate::parse::Document, params :&tower_lsp::lsp_types::CodeActionParams) 
    -> Vec<tower_lsp::lsp_types::CodeActionOrCommand> {
    let mut code_action_respone :Vec<tower_lsp::lsp_types::CodeActionOrCommand> = vec![];
    let range = match document.lsp_range_to_byte_range(&params.range) {
        Some(c) => {c},
        None => {return Vec::new();}
    };
    let mut lt_actions = language_tool::code_actions(client, document, params.text_document.uri.clone(), &range).await;
    code_action_respone.append(&mut lt_actions.0);
    for l in lt_actions.1 {
        CODE_ACTIONS.insert(l.0, l.1);
    }

    code_action_respone
}
pub async fn code_action_resolve(
    params: &tower_lsp::lsp_types::ExecuteCommandParams, 
    backend: &crate::Backend, ) {
    backend.client.log_message(MessageType::INFO, format!("Code action resolve")).await;
    let code_action_ref = match CODE_ACTIONS.get(&params.command) {
        Some(c) => c,
        None => {
            backend.client.log_message(MessageType::INFO, format!("Code action not found, returning")).await;
            return
        }
    };
    let code_action :&CodeActionSource = code_action_ref.deref();
    backend.client.log_message(MessageType::INFO, format!("Code action found")).await;
    match code_action {
        CodeActionSource::LanguageToolCheckText(l) => {
            backend.client.log_message(MessageType::INFO, format!("Check text")).await;
            let (uri, diagnostics) = language_tool::code_action_check_text(backend, l).await.unwrap();
            remove_lsp_diagnostics_of_type(DiagnosticSource::LanguageTool);
            DIAGNOSTICS.lock().unwrap().extend(diagnostics);
            let working_doc_ref = backend.document_map.get(&uri).unwrap();
            let working_doc = working_doc_ref.deref().to_owned();
            send_diagnostics(&backend.client, working_doc, &l.uri).await;
        },
        CodeActionSource::LanguageToolRemoveDiagnostics(_l) => {},
    }
    backend.client.log_message(MessageType::INFO, format!("Returning")).await;

}
pub fn get_lsp_diagnostics(document :&crate::parse::Document) -> Vec<tower_lsp::lsp_types::Diagnostic> {
    let vals: Vec<tower_lsp::lsp_types::Diagnostic> = DIAGNOSTICS
        .lock().unwrap()
        .iter()
        .filter_map(|x| {
            if let Some(c) = x.corrected_diagnostics_lsp(&document) {
            Some(c)
            } else {
            None
            }
        })
    .collect();
    vals
}
pub fn remove_lsp_diagnostics_of_type(typ :DiagnosticSource) {
    DIAGNOSTICS.lock().unwrap().retain(|x| x.source!=typ);
}
