mod config;
mod components;
mod word_query;
mod semantic_token;
mod parse;
use std::ops::{Deref, DerefMut};
use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{LanguageServer, LspService, Server};
use semantic_token::LEGEND_TYPE;
use parse::Backend;
use lazy_static::lazy_static;
use std::process::Command;
use std::env;
use std::path::Path;
use std::sync::RwLock;

lazy_static! {
    static ref WORD_LIST: Vec<String> = word_query::file_to_array("en_wordlist.txt");
    static ref CONFIG: RwLock<config::RootConfig> = RwLock::new(config::RootConfig::default());
}


#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult>{
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "Typst grammar lsp".to_string(),
                version: None
            }),
            offset_encoding: None,
            capabilities: ServerCapabilities {
                inlay_hint_provider: Some(OneOf::Left(true)),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    all_commit_characters: None,
                    work_done_progress_options: WorkDoneProgressOptions{
                        work_done_progress: None,
                    },
                    completion_item: Some(CompletionOptionsCompletionItem {
                        label_details_support: Some(true),
                    }),
                }),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["dummy.do_something".to_string()],
                    work_done_progress_options: Default::default(),
                }),

                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                        SemanticTokensRegistrationOptions {
                            text_document_registration_options: {
                                TextDocumentRegistrationOptions {
                                    document_selector: Some(vec![DocumentFilter {
                                        language: Some("nrs".to_string()),
                                        scheme: Some("file".to_string()),
                                        pattern: None,
                                    }]),
                                }
                            },
                            semantic_tokens_options: SemanticTokensOptions {
                                work_done_progress_options: WorkDoneProgressOptions::default(),
                                legend: SemanticTokensLegend {
                                    token_types: LEGEND_TYPE.into(),
                                    token_modifiers: vec![],
                                },
                                range: Some(true),
                                full: Some(SemanticTokensFullOptions::Bool(true)),
                            },
                            static_registration_options: StaticRegistrationOptions::default(),
                        },
                    ),
                ),
                // definition: Some(GotoCapability::default()),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                ..ServerCapabilities {
                    position_encoding: None,
                    text_document_sync: None,
                    selection_range_provider: None,
                    hover_provider: Some(HoverProviderCapability::Simple(true)),
                    completion_provider: Some(CompletionOptions {
                        resolve_provider: Some(true),
                        trigger_characters: None,
                        work_done_progress_options: WorkDoneProgressOptions{
                            work_done_progress: Some(true)
                        },
                        all_commit_characters: None,
                        completion_item: None
                    }),
                    signature_help_provider: None,
                    definition_provider: None,
                    type_definition_provider: None,
                    implementation_provider: None,
                    references_provider: None,
                    document_highlight_provider: None,
                    document_symbol_provider: None,
                    workspace_symbol_provider: None,
                    code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                    code_lens_provider: None,
                    document_formatting_provider: None,
                    document_range_formatting_provider: None,
                    document_on_type_formatting_provider: None,
                    rename_provider: None,
                    document_link_provider: None,
                    color_provider: None,
                    folding_range_provider: None,
                    declaration_provider: None,
                    execute_command_provider: None,
                    workspace: None,
                    call_hierarchy_provider: None,
                    semantic_tokens_provider: None,
                    moniker_provider: None,
                    inline_value_provider: None,
                    inlay_hint_provider: None,
                    linked_editing_range_provider: None,
                    experimental: None,
                    diagnostic_provider: None,
                }
            },
        })
    }
    async fn initialized(&self, params: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "initialized!")
            .await;
    }
    async fn shutdown(&self) -> Result<()>{
        Ok(())
    }
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.create_document(&params.text_document.uri, params.text_document.version as isize, &params.text_document.text);
        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;
    }
    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        return;
        let text = match params.text {
            Some(c) => {c},
            None => {
                self.client
                    .log_message(MessageType::INFO, "file saved no text!")
                    .await;
                return;
            }
        };
        let uri = &params.text_document.uri;
        self.create_document(uri, 0, &text)
    }
    async fn did_close(&self, _: DidCloseTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file closed!")
            .await;
    }
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = &params.text_document.uri;
        let mut working_doc_ref = match __self.document_map.get_mut(&uri) {
            Some(c) => {c},
            None => {return},
        };
        let working_doc :&mut parse::Document =  working_doc_ref.deref_mut();
        for change in params.content_changes {
            working_doc.change(params.text_document.version, &change);
        }
        components::send_diagnostics(&self.client, working_doc, &uri).await;
    }
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {

        let uri = params.text_document_position_params.text_document.uri;
        let working_doc_ref = match __self.document_map.get(&uri) {
            Some(c) => {c},
            None => {return 
                Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::PlainText,
                        value: "Could not find document, something is really wrong".to_string(),
                    }),
                    range: None
                }))
            },
        };
        let working_doc = working_doc_ref.deref();
        let word = match working_doc.find_word(params.text_document_position_params.position) {
            Some(c) => {c},
            None => {return 
                Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::PlainText,
                        value: "Youre not cursing a word".to_string(),
                    }),
                    range: None
                }))
            },
        };
        let cmd = match Command::new("sdcv").arg("-2").arg("sdcvDict/").arg(word).arg("-e").output() {
            Ok(c) => {c},
            Err(_) => {return 
                Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::PlainText,
                        value: "sdcv didnt not execute correctly, is it installed?".to_string(),
                    }),
                    range: None
                }))
            },
        };
        if !cmd.status.success() {
            return 
                Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::PlainText,
                        value: "CMD did not execute correctly".to_string(),
                    }),
                    range: None
                }))
        }
            // Convert the output bytes to a string
        let output_string = String::from_utf8_lossy(&cmd.stdout);
        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::PlainText,
                value: output_string.to_string(),
            }),
            range: None
        }))
    }
    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        {
            if !CONFIG.read().unwrap().completion_enabled {
                return Ok(None);
            }
        }
        self.client
            .log_message(MessageType::INFO, format!("Completion!"))
            .await;
        let uri = params.text_document_position.text_document.uri;
        let working_doc_ref = match __self.document_map.get(&uri) {
            Some(c) => {c},
            None => {return 
                Ok(None)
            },
        };
        let working_doc = working_doc_ref.deref();
        let mut pos = params.text_document_position.position;
        pos.character-=1;
        let word = match working_doc.find_word(pos) {
            Some(c) => {c},
            None => {return Ok(None)
            },
        };
        let words = word_query::query(&word, &WORD_LIST);
        let mut send_words = String::new();
        for i in &words {
            send_words = format!("{} | {}",send_words, i);
        }
        let mut com_resp :Vec<CompletionItem> = Vec::with_capacity(words.len());
        for w in words {
            com_resp.push(
                CompletionItem {
                    label: w.clone(),
                    label_details: None,
                    kind: Some(CompletionItemKind::VARIABLE),
                    detail: Some(w.clone()),
                    documentation: Some(Documentation::String("Some documentation".to_string())),
                    deprecated: Some(false),
                    preselect: None,
                    sort_text: None,
                    filter_text: None,
                    insert_text: Some(w.clone()),
                    insert_text_format: None,
                    insert_text_mode: Some(InsertTextMode::AS_IS),
                    text_edit: None,
                    additional_text_edits: None,
                    command: None,
                    commit_characters: None,
                    data: None,
                    tags: None,
                }
            )
        }
        return Ok(Some(CompletionResponse::Array(com_resp))) 
    }
    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        self.client.log_message(tower_lsp::lsp_types::MessageType::LOG, "CODE ACTION").await;
        let uri = &params.text_document.uri;
        let working_doc_ref = match __self.document_map.get(&uri) {
            Some(c) => {c},
            None => {return Ok(None)
            },
        };
        let working_doc :&parse::Document = working_doc_ref.deref();
        self.client.log_message(tower_lsp::lsp_types::MessageType::LOG, "COMPONENTS").await;
        let x = components::code_actions(&self.client, working_doc, &params).await;
        Ok(Some(x))
    }
    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<serde_json::Value>> {
        crate::components::code_action_resolve(&params, &self).await;
        Ok(None)
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    {
        let mut config = CONFIG.write().unwrap();
        if let Some(c) = config::RootConfig::init_from_file(Path::new(&args[0])){
            *config = c;
        }
    }

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend {
        client,
        document_map: DashMap::new(),
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
