use lazy_static::lazy_static;
use crate::parse::{Diagnostic, DiagnosticSource, Document};
use std::ops::Range;
use languagetool_rust::{
    check::{
        CheckRequest,
        Replacement, 
        Rule,
    }, 
    server::ServerClient, 
    CheckResponse
};

lazy_static! {
    static ref LT_SERVER_CLIENT :ServerClient = ServerClient::new("http://127.0.0.1", "8081");
}

pub struct LTDiagnostic {
    replacements: Vec<Replacement>,
    rule: Rule,

}
impl Clone for LTDiagnostic {
    fn clone(&self) -> LTDiagnostic {
        LTDiagnostic {
            replacements: self.replacements.clone(),
            rule: self.rule.clone(),
        }
    }
}

pub async fn run_diagnostic(document :&Document, ranges :&Vec<Range<usize>>) 
    -> Option<(Vec<Diagnostic>, Vec<tower_lsp::lsp_types::Diagnostic>)> {
    let mut out_range :Vec<Diagnostic> = vec!{};
    let mut out_lsp_range :Vec<tower_lsp::lsp_types::Diagnostic> = vec!{};

    let mut responses_handler = Vec::new();
    let mut response_range = Vec::new();

    println!("len ranges: {}", ranges.len());
    for range in ranges {
        let text = match document.get_chunk_by_range(range.clone()) {
            Some(c) => c,
            None => {continue;},
        };
        let req = CheckRequest::default()
            .with_text(text);
        let handle = tokio::spawn(async move {
            LT_SERVER_CLIENT.check(&req).await.unwrap()
        });
        response_range.push(range);
        responses_handler.push(handle);
    }
    let response_future = futures::future::try_join_all(responses_handler).await.unwrap();
    let responses :Vec<CheckResponse> = response_future.into_iter().collect();

    let mut out = (vec!{}, vec!{});
    for (i,r) in responses.iter().enumerate() {
        let (a,b) = &mut retrieve_diagnostics(document, response_range[i].start, &r);
        out.0.append( a);
        out.1.append( b);
    }


    Some(out)
}
fn retrieve_diagnostics(document :&Document, start_pos :usize, response :&CheckResponse) -> (Vec<Diagnostic>, Vec<tower_lsp::lsp_types::Diagnostic>) {
    let mut diagnostics :Vec<Diagnostic> = vec!{};
    let mut diagnostics_lsp :Vec<tower_lsp::lsp_types::Diagnostic> = vec!{};
    for m in &response.matches {
        let range = Range{
            start: start_pos+m.offset,
            end: start_pos+m.offset+m.length,
        };
        let lsp_range = match document.byte_range_to_lsp_range(&range) {
            Some(c) => c,
            None => {continue;}
        };

        diagnostics.push(
            Diagnostic {
                range,
                source: DiagnosticSource::LanguageTool(LTDiagnostic {
                    replacements: m.replacements.clone(),
                    rule: m.rule.clone()
                })
            }
        );
        diagnostics_lsp.push(
            tower_lsp::lsp_types::Diagnostic {
                range: lsp_range,
                severity: None,
                code: None,
                code_description: None,
                source: None,
                message: m.message.clone(),
                related_information: None,
                tags: None,
                data: None
            }
            )

    }
    (diagnostics, diagnostics_lsp)
}
