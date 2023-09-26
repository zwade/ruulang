use slang_core::{
    config::config::SlangConfig,
    utils::error::{SlangError, TypecheckError},
    workspace::workspace::Workspace,
};
use tokio::sync::{MappedMutexGuard, Mutex, MutexGuard};
use tower_lsp::{
    jsonrpc,
    lsp_types::{
        Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
        Hover, HoverParams, InitializeParams, InitializeResult, InitializedParams, MessageType,
        ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
    },
    Client, LanguageServer,
};

use crate::utils::location_pair_to_range;

pub struct SlangServer {
    client: Client,

    workspaces: Mutex<Vec<Workspace>>,
}

impl SlangServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            workspaces: Mutex::new(vec![]),
        }
    }

    async fn show_diagnostics(&self, uri: Url, contents: &String, errors: &Vec<SlangError>) {
        let mut diagnostics = vec![];

        for error in errors {
            match error {
                SlangError::TypecheckError(TypecheckError::GeneralError(data)) => {
                    if data.loc.is_none() {
                        continue;
                    }

                    let loc = data.loc.unwrap();

                    diagnostics.push(Diagnostic::new(
                        location_pair_to_range(&contents, loc.0 as u32, loc.1 as u32),
                        Some(DiagnosticSeverity::ERROR),
                        None,
                        None,
                        data.data.clone(),
                        None,
                        None,
                    ))
                }
                SlangError::SlangParseError(location) => diagnostics.push(Diagnostic::new(
                    location_pair_to_range(&contents, *location as u32, *location as u32 + 1),
                    Some(DiagnosticSeverity::ERROR),
                    None,
                    None,
                    "Parse error".to_string(),
                    None,
                    None,
                )),
                _ => {}
            }
        }

        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    async fn workspace_for_file(&self, file_uri: &Url) -> Option<MappedMutexGuard<'_, Workspace>> {
        let file_name = file_uri.to_file_path().unwrap();

        let workspace_index = self
            .workspaces
            .lock()
            .await
            .iter()
            .position(|workspace| workspace.contains_file(&file_name))?;

        let workspaces = self.workspaces.lock().await;
        let result = MutexGuard::map(workspaces, |workspaces| &mut workspaces[workspace_index]);

        Some(result)
    }

    async fn patch_and_notify(&self, file_uri: &Url, contents: &String, allow_create: bool) {
        let file_path = file_uri.to_file_path();
        if file_path.is_err() {
            // We're just editing an unsaved buffer
            return;
        }

        let file_name = file_path.unwrap();
        let maybe_workspace = self.workspace_for_file(&file_uri).await;

        if let Some(mut workspace) = maybe_workspace {
            if !allow_create && !workspace.contains_file(&file_name) {
                self.client
                    .log_message(
                        MessageType::INFO,
                        format!("Skipping file: {:#?}", &file_name),
                    )
                    .await;

                return;
            }

            let success = workspace.patch_file(&file_name, contents).await;
            match success {
                Ok(_) => {
                    let contents = &contents;
                    let errors = workspace.typecheck_file(&file_name).await;

                    self.client
                        .log_message(
                            MessageType::INFO,
                            format!("Typechecking {:#?}: {:#?}", &file_name, &errors),
                        )
                        .await;

                    self.show_diagnostics(file_uri.clone(), &contents, &errors)
                        .await;
                }
                Err(err) => {
                    self.client
                        .log_message(MessageType::ERROR, format!("{:#?}", err))
                        .await;
                }
            }
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for SlangServer {
    async fn initialize(&self, params: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        let mut workspaces = vec![];

        if let Some(folders) = params.workspace_folders {
            for folder in folders {
                let root_dir = folder.uri.to_file_path().unwrap();
                let file_path = root_dir.join("slang.toml");
                let config = SlangConfig::load(&file_path, &root_dir).await;

                if let Err(err) = config {
                    self.client
                        .log_message(MessageType::ERROR, format!("{:#?}", err))
                        .await;
                    continue;
                }

                let workspace = Workspace::new(config.unwrap(), root_dir);
                workspaces.push(workspace)
            }
        }

        self.workspaces.lock().await.extend(workspaces);

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                // hover_provider: Some(HoverProviderCapability::Simple(true)),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),

                ..Default::default()
            },

            ..Default::default()
        })
    }

    async fn initialized(&self, params: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let file_uri = &params.text_document.uri;
        let contents = &params.text_document.text;

        if params.text_document.language_id != "slang" {
            return;
        }

        self.patch_and_notify(file_uri, contents, true).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let file_uri = &params.text_document.uri;
        let contents = &params.content_changes[0].text;

        self.patch_and_notify(file_uri, contents, false).await;
    }

    async fn hover(&self, params: HoverParams) -> jsonrpc::Result<Option<Hover>> {
        self.client
            .log_message(MessageType::INFO, format!("Hover: {:#?}", params))
            .await;

        let file_uri = params.text_document_position_params.text_document.uri;
        let file_name = file_uri.to_file_path().unwrap();
        let maybe_workspace = self.workspace_for_file(&file_uri).await;

        let workspace = match maybe_workspace {
            Some(workspace) => workspace,
            None => return Ok(None),
        };

        let contents = match workspace.resolve_schema(&file_name) {
            Some(data) => data,
            None => return Ok(None),
        };

        let _file = match &contents.data {
            Ok(file) => file,
            Err(_) => return Ok(None),
        };

        Ok(None)
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        Ok(())
    }
}
