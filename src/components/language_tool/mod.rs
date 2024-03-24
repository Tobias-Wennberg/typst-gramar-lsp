

/* Most of this code is copied from https://github.com/antonWetzel/typst-languagetool.git
 * with small modifications
*/


mod check_text;
use std::collections::HashMap;
use languagetool_rust::check::{
        Replacement, 
        Rule,
    };
use tower_lsp::lsp_types::MessageType;
use tower_lsp::lsp_types::TextEdit;
use tower_lsp::lsp_types::Url;
use tower_lsp::lsp_types::WorkspaceEdit;
use crate::components::Diagnostic;
use uuid::Uuid;
use std::ops::Range;
use std::ops::Deref;
use std::clone::Clone;

#[derive(Clone)]
pub struct LTDiagnostic {
    pub replacements: Vec<Replacement>,
    pub rule: Rule,

}
pub struct LTCodeActionCheckText {
    pub uri :Url,
}
pub struct LTCodeActionRemoveDiagnostic {
    uri :Url,
}
pub struct TextCheck {
    pub diagnostic: Vec<Diagnostic>,
    pub matches: Vec<MatchChunk>,

}
pub struct MatchChunk {
    matches: Vec<Match>,
    language_code: String
}
pub struct Match {
    match_data: languagetool_rust::check::Match,
}

pub async fn check(document :&crate::parse::Document) -> Vec<Diagnostic> {
    check_text::check(document).await.0
}
pub async fn code_action_check_text(backend :&crate::Backend, values :&LTCodeActionCheckText)
    -> Option<(Url, Vec<Diagnostic>)> {
    let working_doc_ref = match backend.document_map.get(&values.uri.clone()) {
        Some(c) => {c},
        None => {return None}
    };
    let working_doc = working_doc_ref.deref().to_owned();
    backend.client.show_message(tower_lsp::lsp_types::MessageType::LOG, "Laddar med language tools".to_string()).await;
    let checks = check(&working_doc).await;
    Some((values.uri.clone(), checks))
}
pub async fn code_actions(client :&tower_lsp::Client, document :&crate::parse::Document, uri :Url, range :&Range<usize>) 
    -> (Vec<tower_lsp::lsp_types::CodeActionOrCommand>, Vec<(String, crate::components::CodeActionSource)> ) {
    let hovering_error :Vec<crate::components::Diagnostic> =  crate::components::DIAGNOSTICS
        .lock().unwrap()
        .iter()
        .filter_map( |x| {
            let x_range = match document.correct_range(x.version.clone(), x.range.clone()) {
                Some(c) => c,
                None => return None,
            };
            if x.source == crate::components::DiagnosticSource::LanguageTool
                && x_range.end >= range.end
                && x_range.start <= range.start
            {
                Some(x.clone()) // Assuming you want to include the diagnostic itself
            } else {
                None
            }
        })
        .collect();
    client.log_message(MessageType::INFO, format!("Hovering error length: {}", hovering_error.len())).await;
    client.log_message(MessageType::INFO, format!("Range: start: {}, stop: {}", range.start, range.end)).await;
    
    let uuid1 = Uuid::new_v4().to_string();
    let uuid2 = Uuid::new_v4().to_string();
    let mut tower_lsp_diagnostics = vec![
        tower_lsp::lsp_types::CodeActionOrCommand::Command(tower_lsp::lsp_types::Command {
            title: "Evaluate with language tools".to_string(),
                command: uuid1.clone(),
                arguments: None,
            }),
        tower_lsp::lsp_types::CodeActionOrCommand::Command(tower_lsp::lsp_types::Command {
            title: "Remove language tools diagnostics".to_string(),
                command: uuid2.clone(),
                arguments: None,
            }),
    ];
    let component_diagnostic = vec![
        (
            uuid1,
            crate::components::CodeActionSource::LanguageToolCheckText(LTCodeActionCheckText {
                uri: uri.clone()
            })
        ),
        (
            uuid2,
            crate::components::CodeActionSource::LanguageToolRemoveDiagnostics(LTCodeActionRemoveDiagnostic {
                uri: uri.clone()
            })
        )
    ];
    for err in hovering_error {
        tower_lsp_diagnostics.extend(
            diagnostic_code_action(document, &err, &uri)
            )
    }
    (tower_lsp_diagnostics, component_diagnostic)
}
fn diagnostic_code_action(document :&crate::parse::Document, diagnostic :&crate::components::Diagnostic, uri :&Url) 
    -> Vec<tower_lsp::lsp_types::CodeActionOrCommand> {
    let lt_dia :LTDiagnostic = match &diagnostic.source_data {
        super::DiagnosticSourceData::LanguageTool(c) => c.clone(),
        _ => {return Vec::new()}
    };
    let mut return_var :Vec<tower_lsp::lsp_types::CodeActionOrCommand> = Vec::new();
    let corrected_range = match document.correct_range(diagnostic.version, diagnostic.range.clone()) {
        Some(c) => {c},
        None => {return Vec::new();},
    };
    let lsp_range = match document.byte_range_to_lsp_range(&corrected_range) {
        Some(c)  => {c},
        None => {return Vec::new();}
    };
    let lsp_diagnostics :Vec<tower_lsp::lsp_types::Diagnostic> = vec![
        diagnostic.corrected_diagnostics_lsp(&document).unwrap()
    ];
    for lt_replacement in lt_dia.replacements {
        let mut replacement :HashMap<tower_lsp::lsp_types::Url, Vec<TextEdit>> = HashMap::new();
        replacement.insert(uri.clone(), vec![
            tower_lsp::lsp_types::TextEdit {
                range: lsp_range.clone(),
                new_text: lt_replacement.value.clone()
            }
        ]);

        return_var.push(
            tower_lsp::lsp_types::CodeActionOrCommand::CodeAction(tower_lsp::lsp_types::CodeAction {
                title: format!("Rule: {} | Replacement: {}", lt_dia.rule.description.clone(), lt_replacement.value.clone()).to_string(),
                kind: None,
                diagnostics: Some(lsp_diagnostics.clone()),
                edit: Some( WorkspaceEdit {
                    changes: Some(replacement),
                    document_changes: None,
                    change_annotations: None,
                }),
                command: None,
                is_preferred: None,
                disabled: None,
                data: None
            })
        );

    }
    return_var

}
